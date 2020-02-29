// this module will contain protocol definitions
// and core data types and handlers for IRC commands
//use crate::parser;

pub mod rfc_defs;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::clone::Clone;
use crate::client;
use crate::client::{Client,ClientList,ClientType};
use crate::parser::ParsedMsg;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use dns_lookup::lookup_addr;
use tokio::net::TcpStream;

// I hope it doesnt get too confusing that parser.rs and irc.rs both have a 'Host' enum,
// main difference is the parser's variants only contain strings (either for hostname
// or ip address), the type here will contain a hostname string or a proper ip address type
// from std::net
pub enum Host {
    Hostname(String),
    HostAddr(IpAddr)
}

pub enum Origin {
    Server(Host),
    User(String, Option<String>, Option<Host>)
}

// having to define and match these types here,
// as well as matching the raw command strings in parser::parse_message()
// feels somehow not quite right, but I haven't yet got a more elegant solution
// maybe parser should just hand over an Option<String> containing the src/origin prefix,
// followed by a vector reference of String args
// but the construction of the enum / match case is still clumsy,
// I want an elegant and fast way to go from a string literal to an enum...
// even with a match to string literals, I think it makes more sense to
// have the specific command cases handled here, and not have to change lots of things both
// here and in parser() if we want to add/change how commands work
pub enum Command {
    Join(Vec<String>), // #channel
    Part(Vec<String>, Option<String>), // #channel, part-message
    Message(Vec<String>, String), // dest (user/#channel), message
    Nick(String), // choose nickname
    User(String, u64, String), // choose username (might need addition parameters)
    Quit(Option<String>), // quit-message
}

pub struct UserFlags {
    registered: bool,
}

// of the above, Message(targets, conntent) is arguably the most important feature of an irc server
// it's also emblematic of the way we need to think about structuring data and finding targets,
// there are a few cases we need to consider
// if target is a channel, we have to map that to a list of users JOINed to that channel,
// this can be done with the Channel::users field, but to get to the Channel data struct,
// we need a HashMap of channel names in order to locate that data
// once we have converted our list of targets to a list of nicks, we need to check which of those
// are local (connected as clients to our server), and which are remote - for those 
// which are remote, we have to remove their names from the target list and substitute in a server,
// so we need to know which server acts as a relay for each remote user
pub struct User {
    id: u64,                            // we can have this just be the same as the client_id
    nick: String,
    username: String,
    real_name: String,
    host: Host,
    channel_list: Vec<String>,
    flags: UserFlags
}

pub struct ProtoUser {
    nick: Option<String>,
    username: Option<String>,
    real_name: Option<String>
}

pub struct ServerUserFlags {
    server_op: bool
}

pub struct ServerUser {
    nick: String,
    flags: ServerUserFlags
}

// client may need to be mutable, and probably needs some sort of lifetime parameter
// this is OK as long as the Client is not borrowed anywhere else
// alternatively, we once again use an id as a place holder for a borrow,
// and the core contains a hashmap of client IDs
pub struct Server {
    id: u64,
    host: Host,
    users: Vec<ServerUser>,
    client_id: u64             // need to map 'server' to socketed client
}

//pub struct Message {
//    pub content: Box<Command>,// commands and their parameters are defined as an enum above
//    pub origin: Option<Origin> // important when servers relay commands originating from their users
//}

// flags related to user priveleges in a specific channel
pub struct ChanUserFlags {
    chan_op: bool,
    chan_halfop: bool,
    chan_voice: bool
}

pub struct ChanUser {
    nick: String,
    flags: ChanUserFlags
}

// channel needs a name, a topic, and a list of joined users
// this list can't just be a list of nicks, as additional flags are required: is the user an op on
// the channel, for example?
pub struct Channel {
    name: String,
    users: Vec<ChanUser>,
    topic: String
}

// when we want to do all this with concurrency,
// we could wrap each HashMap in a Mutex, while the
// Core itself will just have immutable borrows
// when a hashmap has been used to obtain a User, Channel or Server,
// again a Mutex wrapper will be needed, and used for as short a time
// as possible in every case
// commands probably doesn't need a Mutex, it will be populated once,
// then remain the same
// do we also need to wrap these in Arc<T> pointers? :/
// maybe it's possible just to have the Core in an Arc<T>,
// and give each thread a pointer to the core?
pub struct Core {
    pub clients: Arc<Mutex<ClientList>>,                    // maps client IDs to clients
    pub nicks: Arc<Mutex<HashMap<String, u64>>>,                    // maps nicknames to unique ids
    pub users: Arc<Mutex<HashMap<u64, Arc<Mutex<User>>>>>,          // maps user IDs to users
    pub channels: Arc<Mutex<HashMap<String, Arc<Mutex<Channel>>>>>, // maps channames to Channel data structures
    pub servers: Arc<Mutex<HashMap<u64, Arc<Mutex<Server>>>>>,      // maps server IDs to servers
    pub commands: Arc<Mutex<HashMap<String, Arc<Mutex<Command>>>>>  // map of commands
}

// init hash tables
// let's have this copyable, so whatever thread is doing stuff,
// needs to only mutex lock one hashmap at a time
impl Core {
    pub fn new () -> Self {
        // initialize hash tables for clients, servers, commands
        // clones of the "irc Core" are passed as a field within
        // ClientFuture, but we can still have a client list within
        // irc::Core, as Client is a seperate element within ClientFuture,
        // so we still avoid cyclic refs
        let clients = Arc::new(Mutex::new(ClientList::new()));
        let nicks = Arc::new(Mutex::new(HashMap::new()));
        let commands = Arc::new(Mutex::new(HashMap::new()));
        let servers  = Arc::new(Mutex::new(HashMap::new()));
        let users = Arc::new(Mutex::new(HashMap::new()));
        let channels = Arc::new(Mutex::new(HashMap::new()));
        Core {
            clients,
            nicks,
            commands,
            channels,
            users,
            servers
        }
    }
}

impl Clone for Core {
    fn clone (&self) -> Self {
        Core {
            clients: Arc::clone(&self.clients),
            nicks: Arc::clone(&self.nicks),
            commands: Arc::clone(&self.commands),
            channels: Arc::clone(&self.channels),
            users: Arc::clone(&self.users),
            servers: Arc::clone(&self.servers)
        }
    }
}

// handle command should take a Client and a ParseMsg
// the command string will be converted to uppercase and a match block
// will redirect to the specific command handler
pub fn handle_command (core: &mut Core, client: &mut Client, params: ParsedMsg) {
    // we're matching a String to some &str literals, so may need this &
    match &params.command[..] {
        "NICK" => cmd_nick(&mut client, params), // <-- will the borrow checker hate me for this? let's see...
//        "USER" => cmd_user(&mut client, params) //      possibly not, since it's immutable and passed-ownership
    }
}

fn cmd_nick(client: &mut Client, params: ParsedMsg) {
    let args: Vec<String>;
    if let Some(args) = params.opt_params {
        match client.client_type { // I think maybe the borrow checker will hate me for reassigning client_type within its own match block
            ClientType::Unregistered => { // in this case we need to create a "proto user"
                client.client_type = ClientType::ProtoUser(Arc::new(Mutex::new(ProtoUser {
                    nick: Some(args[0]),
                    username: None,
                    real_name: None })));
                client.send_line("created a proto user thingy :o");
            },
            ClientType::User(user_ref) => { // just a nick change
                let user = user_ref.lock().unwrap();
                user.nick = args[0];
            },
            ClientType::ProtoUser(proto_user_ref) => { // in this case we already got USER
                let proto_user = proto_user_ref.lock().unwrap();
                let username = proto_user.username.unwrap();
                let real_name = proto_user.real_name.unwrap();
                // now we need to create a real User for the client
                if let Ok(address) = client.socket.peer_addr() {
                    let host = if let Ok(h) = lookup_addr(&address) {
                        h
                    } else {
                        address.to_string()
                    };
                    client.client_type = ClientType::User(Arc::new(Mutex::new(User {
                        id: client.id,
                        nick: args[0],
                        username,
                        real_name,
                        host: asdf,
                        channel_list: Vec::new(),
                        flags: UserFlags { registered: true }
                    })));
                } else {
                    client.dead = true;
                }
            }
            ClientType::Server(server_ref) => return,
        }
    } else {
        client.send_line("not enough parameters!");
    }
}
