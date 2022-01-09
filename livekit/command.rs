use tiny_tokio_actor::Message;
use crate::config::GroupConfig;

pub type CommandResult<T> = Result<T, String>;

macro_rules! command {
    {
        $name:ident {$($field:ident: $t:ty $(,)?)*} => $rtype:ty
    } => {
        #[derive(Clone)]
        pub struct $name {
            $(pub $field: $t),*
        }

        impl Message for $name {
            type Response = CommandResult<$rtype>;
        }
    };
    {
        $name:ident => $rtype:ty
    } => {
        #[derive(Clone)]
        pub struct $name;

        impl Message for $name {
            type Response = CommandResult<$rtype>;
        }
    };
}

command! { AddRooms { msroomids: Vec<i64> } => () }

command! { ActivateRooms { roomids: Vec<u32> } => () }

command! { InactivateRooms { roomids: Vec<u32> } => () }

command! { DropRooms { roomids: Vec<u32> } => () }

command! { DumpConfig => GroupConfig }

command! { DumpStatus => String }

command! { CloseAll => () }
