// This is not how you define enums, should be Variant(type, type),
// but this might serve as useful boilerplate later to turn into a
// static lookup table for the associated messages...
// &'static would probably be fine I think
use std::{error, fmt};
type NumReply = usize;

impl error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::NoSuchNick(n, txt) => write!(f, "{} {}", n, txt),
            Error::NoRecipient(n, txt) => write!(f, "{} {}", n, txt),
            Error::NoTextToSend(n, txt) => write!(f, "{} {}", n, txt),
            Error::UnknownCommand(n, txt) => write!(f, "{} {}", n, txt),
            Error::NicknameInUse(n, txt) => write!(f, "{} {}", n, txt),
            Error::NotRegistered(n, txt) => write!(f, "{} {}", n, txt),
            Error::NeedMoreParams(n, txt) => write!(f, "{} {}", n, txt),
            Error::AlreadyRegistred(n, txt) => write!(f, "{} {}", n, txt),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    NoSuchNick(          NumReply, &'static str),
//    NoSuchServer(        NumReply, &'static str),
//    NoSuchChannel(       NumReply, &'static str),
//    CannotSendToChan(    NumReply, &'static str),
//    TooManyChannels(     NumReply, &'static str),
//    WasNoSuchNick(       NumReply, &'static str),
//    TooManyTargets(      NumReply, &'static str),
//    NoOrigin(            NumReply, &'static str),
    NoRecipient(         NumReply, &'static str),
    NoTextToSend(        NumReply, &'static str),
//    NoTopLevel(          NumReply, &'static str),
//    WildTopLevel(        NumReply, &'static str),
    UnknownCommand(      NumReply, &'static str),
//    NoMotd(              NumReply, &'static str),
//    NoAdminInfo(         NumReply, &'static str),
//    FileError(           NumReply, &'static str),
//    NoNickNameGiven(     NumReply, &'static str),
//    ErroneusNickname(    NumReply, &'static str),
    NicknameInUse(       NumReply, &'static str),
//    NickCollision(       NumReply, &'static str),
//    UserNotInChannel(    NumReply, &'static str),
//    NotOnChannel(        NumReply, &'static str),
//    UserOnChannel(       NumReply, &'static str),
//    NoLogin(             NumReply, &'static str),
//    SummonDisabled(      NumReply, &'static str),
//    UsersDisabled(       NumReply, &'static str),
    NotRegistered(       NumReply, &'static str),
    NeedMoreParams(      NumReply, &'static str),
    AlreadyRegistred(    NumReply, &'static str),
//    NoPermForHost(       NumReply, &'static str),
//    PasswdmisMatch(      NumReply, &'static str),
//    YoureBannedCreep(    NumReply, &'static str),
//    KeySet(              NumReply, &'static str),
//    ChannelIsFull(       NumReply, &'static str),
//    UnknownMode(         NumReply, &'static str),
//    InviteOnlyChan(      NumReply, &'static str),
//    BannedFromChan(      NumReply, &'static str),
//    BadChannelKey(       NumReply, &'static str),
//    NoPrivileges(        NumReply, &'static str),
//    ChanOPrivsNeeded(    NumReply, &'static str),
//    CantKillServer(      NumReply, &'static str),
//    NoOperHost(          NumReply, &'static str),
//    UModeUnknownFlag(    NumReply, &'static str),
//    UsersDontMatch(      NumReply, &'static str),
}

pub const ERR_NOSUCHNICK: Error = Error::NoSuchNick(   401, "<nickname> :No such nick/channel");
//pub const ERR_: Error = NoSuchServer(        402, "<server name> :No such server"),
//pub const ERR_: Error = NoSuchChannel(       403, "<channel name> :No such channel"),
//pub const ERR_: Error = CannotSendToChan(    404, "<channel name> :Cannot send to channel"),
//pub const ERR_: Error = TooManyChannels(     405, "<channel name> :You have joined too many channels"),
//pub const ERR_: Error = WasNoSuchNick(       406, "<nickname> :There was no such nickname"),
//pub const ERR_: Error = TooManyTargets(      407, "<target> :Duplicate recipients. No message delivered"),
//pub const ERR_: Error = NoOrigin(            409, ":no origin specified"),
pub const ERR_NORECIPIENT: Error = Error::NoRecipient(         411, ":No recipient given (<command>)");
pub const ERR_NOTEXTTOSEND: Error = Error::NoTextToSend(        412, ":No text to send");
//pub const ERR_: Error = NoTopLevel(          413, "<mask> :No toplevel domain specified"),
//pub const ERR_: Error = WildTopLevel(        414, "<mask> :Wildcard in toplevel domain"),
pub const ERR_UNKNOWNCOMMAND: Error = Error::UnknownCommand(      421, "<command> :Unknown command");
//pub const ERR_: Error = NoMotd(              422, ":MOTD File is missing"),
//pub const ERR_: Error = NoAdminInfo(         423, "<server> :No administrative info available"),
//pub const ERR_: Error = FileError(           424, ":File error doing <file op> on <file>"),
//pub const ERR_: Error = NoNickNameGiven(     431, ":No nickname given"),
//pub const ERR_: Error = ErroneusNickname(    432, "<nick> :Erroneus nickname"),
pub const ERR_NICKNAMEINUSE: Error = Error::NicknameInUse(       433, "<nick> :Nickname is already in use");
//pub const ERR_: Error = NickCollision(       436, "<nick> :Nickname collision KILL"),
//pub const ERR_: Error = UserNotInChannel(    441, "<nick> <channel> :They aren't on that channel"),
//pub const ERR_: Error = NotOnChannel(        442, "<channel> :You're not on that channel"),
//pub const ERR_: Error = UserOnChannel(       443, "<user> <channel> :is already on channel"),
//pub const ERR_: Error = NoLogin(             444, "<user> :User not logged in"),
//pub const ERR_: Error = SummonDisabled(      445, ":SUMMON has been disabled"),
//pub const ERR_: Error = UsersDisabled(       446, ":USERS has been disabled"),
pub const ERR_NOTREGISTERED: Error = Error::NotRegistered(       451, ":You have not registered");
pub const ERR_NEEDMOREPARAMS: Error = Error::NeedMoreParams(      461, "<command> :Not enough parameters");
pub const ERR_ALREADYREGISTRED: Error = Error::AlreadyRegistred(    462, ":You may not reregister");
//pub const ERR_: Error = NoPermForHost(       463, ":Your host isn't among the privileged"),
//pub const ERR_: Error = PasswdmisMatch(      464, ":Password incorrect"),
//pub const ERR_: Error = YoureBannedCreep(    465, ":You are banned from this server"),
//pub const ERR_: Error = KeySet(              467, "<channel> :Channel key already set"),
//pub const ERR_: Error = ChannelIsFull(       471, "<channel> :Cannot join channel (+l)"),
//pub const ERR_: Error = UnknownMode(         472, "<char> :is unknown mode char to me"),
//pub const ERR_: Error = InviteOnlyChan(      473, "<channel> :Cannot join channel (+i)"),
//pub const ERR_: Error = BannedFromChan(      474, "<channel> :Cannot join channel (+b)"),
//pub const ERR_: Error = BadChannelKey(       475, "<channel> :Cannot join channel (+k)"),
//pub const ERR_: Error = NoPrivileges(        481, ":Permission Denied- You're not an IRC operator"),
//pub const ERR_: Error = ChanOPrivsNeeded(    482, "<channel> :You're not channel operator"),
//pub const ERR_: Error = CantKillServer(      483, ":You cant kill a server!"),
//pub const ERR_: Error = NoOperHost(          491, ":No O-lines for your host"),
//pub const ERR_: Error = UModeUnknownFlag(    501, ":Unknown MODE flag"),
//pub const ERR_: Error = UsersDontMatch(      502, ":Cant change mode for other users")