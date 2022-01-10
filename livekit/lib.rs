macro_rules! command {
    {
        $name:ident {$($field:ident: $t:ty $(,)?)*} => $rtype:ty
    } => {
        #[derive(Clone)]
        pub struct $name {
            $(pub $field: $t),*
        }

        impl tiny_tokio_actor::Message for $name {
            type Response = CommandResult<$rtype>;
        }
    };
    {
        $name:ident => $rtype:ty
    } => {
        #[derive(Clone)]
        pub struct $name;

        impl tiny_tokio_actor::Message for $name {
            type Response = CommandResult<$rtype>;
        }
    };
}

#[derive(Clone)]
pub struct GlobalEvent(String);

impl tiny_tokio_actor::SystemEvent for GlobalEvent {}

pub mod config;
pub mod group;
