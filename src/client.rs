// client
// this file contains futures, handlers and socket code for dealing with
// async IO for connected clients
extern crate tokio;
extern crate futures;

use crate::buffer;
use crate::irc;
use crate::parser;

use std::sync::{Mutex, Arc};
use std::net::SocketAddr;
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncRead, AsyncWrite};
use futures::{Future, Async, Poll, Stream};
use futures::task;
use futures::task::Task;
use crate::buffer::MessageBuffer;
use crate::irc::rfc_defs as rfc;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub enum ClientCommand {
    Empty
}

// this future is a wrapper to the Client struct, and implements the polling code
struct ClientFuture {
    client: Arc<Mutex<Client>>,
    client_list: Arc<Mutex<ClientList>>,
    id: u32, // same as client id
    first_poll: bool,
}

pub struct Client { // is it weird/wrong to have an object with the same name as the module?
    // will need a hash table for joined channels
    //channels: type unknown
    socket: TcpStream,
    //flags: some sort of enum vector?
    //host: irc::Host,
    id: u32,
    input: MessageBuffer,
    output: MessageBuffer,
    handler: Task,
    dead: bool // this will be flagged if poll() needs to remove the client
}

struct ClientList {
    map: HashMap<u32, Arc<Mutex<Client>>>,
    next_id: u32
}

impl Future for ClientFuture {
    type Item = ();
    type Error = ();

    // call when client connection drops (either in error or if eof is received)
    fn drop(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut client_list = self.client_list.lock().unwrap();
        client_list.map.remove(&client.id);
        return Ok(Async::Ready(()));
    }
    
    // to be called from polling future
    fn try_flush(&mut self, client: &mut Client) -> Poll<Self::Item, Self::Error>  {
        // now we also have the slightly annoying situation that if bytes_out < out_i,
        // we have to either do someething complicated with two indices, or shift
        // bytes to the start of the buffer every time a write completes
        let mut write_count: usize = 0;
        let mut tmp_buf: [u8; rfc::MAX_MSG_SIZE] = [0; rfc::MAX_MSG_SIZE];

        // create a special scope where we use the Arc<Mutex<>> wrapper to copy stuff into 
        // our temporary write buffer
        let len = client.output.copy(&mut tmp_buf); // returns bytes copied

        // write as much as we can while just incrementing indices
        while write_count < len {
            match client.socket.poll_write(&tmp_buf[write_count .. len]) {
                Ok(Async::Ready(bytes_out)) => write_count += bytes_out, // track how much we've written
                Ok(Async::NotReady) => break,
                Err(e) => return Err(e)
            }
        }

        // if write_count > 0, get mutex again and shift bytes
        // (or just reset index if write_count == index
        if write_count > 0 {
            if write_count == client.output.index {
                client.output.index = 0;
            } else {
                client.output.shift_bytes_to_start(write_count);
            }
        }

        // only return Ready when it's time to drop the client
        return Ok(Async::NotReady);
    }

    fn try_read(&mut self, client: &mut Client) -> Poll<Self::Item, Self::Error> {
        // we'll read anything we can into a temp buffer first, then only later
        // transfer it to the mutex guarded client.output buffer
        let mut tmp_buf: [u8; rfc::MAX_MSG_SIZE] = [0; rfc::MAX_MSG_SIZE];
        let mut tmp_index: usize = 0;
        while tmp_index < rfc::MAX_MSG_SIZE { // loop until there's nothing to read or the buffer's full
            match client.socket.poll_read(&mut tmp_buf[tmp_index ..]) {
                Ok(Async::Ready(bytes_read)) if bytes_read == 0 =>
                    // EOF - this Future completes when the client is no more
                    return Ok(Async::Ready(())),
                Ok(Async::Ready(bytes_read)) => tmp_index += bytes_read,
                Ok(Async::NotReady) => return Async::NotReady,
                Err(e) => return Err(e)
            }
        }

        // now we have (potentially) filled some bytes in a temp buffer
        // get a mutex lock and update stuff
        if tmp_index > 0 {
            // if the below call returns an error, the client will be dropped
            client.input.append_bytes(&mut tmp_buf[.. tmp_index])?;
        }
    }

    // forward incoming message to other users
    fn broadcast(&mut self, map: &mut HashMap<u32, Arc<Mutex<Client>>>, msg: &str) {
        for (id, other_client) in &client_list.map {
            // skip writing to ourself
            if *id == self.id {
                continue;
            }
            
            // get a mutex on the other client
            let mut other_client = other_client.lock().unwrap();

            // send_line() takes care of notifying the Future and flags
            // the client as dead if the append fails (indicates full buffer
            // (indicates flushes are not successful))
            other_client.send_line(&msg);
        }
    }

    // this here is the main thing
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let client = Arc::clone(&self.client);
        let mut client = client.lock().unwrap();  // client is now under mutex lock

        // is connection/client dead? drop from list and return Ready to complete our future
        if client.dead == true {
            return self.drop(&mut client);
        }

        // if its the first time polling, we need to register our task
        if self.first_poll == true {
            self.first_poll = false;
            client.handler = task::current();
        }

        // try to write if there is anything in outbuf,
        // returns error if there is a connection problem, in which case drop the client
	if let Err(_e) = self.try_flush(&mut client) {
            return self.drop(&mut client);
        }

        // try to read into our client's in-buffer
        match self.try_read(&mut client) {
            Ok(Async::NotReady) => (), // do nothing
            Ok(Async::Ready(_)) => return self.drop(&mut client), // EOF
            Err(_e) => return self.drop(&mut client) // Connection error
        }
                    
        // loop while client's input buffer contains line delimiters
        let client_list = self.client_list.lock().unwrap();
        while client.input.has_delim() {
            let msg_string = inbuf.extract_ln();
            self.broadcast(&client_list, msg_string);
        }
        Ok(Async::NotReady)
    }
}

// externally, clients will probably be collected in a vector/hashmap somewhere
// each client will have a unique identifier: their host (type irc::Host),
// each must have a socket associated with it
// clients here mean something associated with a socket connection -
// i.e. they can be servers or users
// somewhere we'll need code for mapping external users to whatever
// relay server we can reach them through
impl Client {
    // since new clients will be created on a new connection event,
    // we'll need a socket type as a parameter
    // implementation decision: explicitly return as a pointer to heap data
    // will also be necessary that all threads can access every client object
    pub fn new(id: u32, task: Task, socket: TcpStream) -> Client {
        Client {
            output: buffer::MessageBuffer::new(),
            input: buffer::MessageBuffer::new(),
            handler: task, // placeholder
            dead: false,
            socket,
            id
        }
    }

    // an event handler waiting on new data from the client
    // must call this handler when a CR-LF is found
    // return type is a ClientCommand, which will be processed elsewhere
    pub fn end_of_line(&mut self) -> Result<ClientCommand, parser::ParseError> {
        // NB: buffer index might not be directly after the CR-LF
        // first bit of code locates a CR-LF
        // next bit extracts a string and moves buffer data after CR-LF
        // to front, reseting the index afterwards
        let command_string = self.inbuf.extract_ln();

        // i will insist that the event handler doesn't hand us empty lines
        assert!(command_string.len() != 0);
        let parsed_msg = parser::parse_message(&command_string)?;

        // do something with the parsed message, irc.rs code needs to get involved
        Ok(ClientCommand::Empty)
    }

    // fn sends a line to the client - this function adds the cr-lf delimiter,
    // so just an undelimited line should be passed as a &str
    // the function also notifies the runtime that the socket handler needs
    // to be polled to flush the write
    pub fn send_line(&mut self, buf: &str) {
        let mut outbuf = self.output.lock().unwrap();
        let mut string = String::from_utf8_lossy(&buf).to_string();
        string.push_str("\r\n");
        self.handler.notify();
        if let Err(_e) = output.append_str(string) {
            self.dead = true;
        }
    }
}
