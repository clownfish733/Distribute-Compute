#[macro_export]
macro_rules! wire_message {
    ($enum_name:ident {
        $($variant:ident($type:ty) = $id:expr), * $(,)?
    }) => {
        impl $enum_name {
            fn type_id(&self) -> u8 {
                match self {
                    $(
                        $enum_name::$variant(_) => $id,
                    )*
                }
            }

            fn to_wire(&self) -> Result<Vec<u8>, P2PError>
            where
                $($type: WireEncodable,)*
            {
                match self {
                    $(
                        $enum_name::$variant(msg) => msg.encode(),
                    )*
                }
            }

            fn from_wire(type_id: u8, payload: &[u8]) -> Result<Self, P2PError>
            where
                $($type: WireEncodable,)*
            {
                match type_id {
                    $(
                        $id => Ok($enum_name::$variant(<$type>::decode(payload)?)),
                    )*
                    _ => Err(P2PError::UnknownMessageType(type_id)),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! message_dispatch {
    ($($variant:ident($type:ty) = $category:expr), * $(,)?) => {
        impl Message {
            pub fn encode(&self) -> Result<Vec<u8>, P2PError> {
                match self {
                    $(
                        Message::$variant(msg) => msg.to_wire(),
                    )*
                }
            }

            pub fn message_type(&self) -> u32 {
                match self {
                    $(
                        Message::$variant(msg) => {
                            ($category as u32) | (msg.type_id() as u32)
                        }
                    )*
                }
            }

            pub fn decode(msg_type: u32, payload: &[u8]) -> Result<Self, P2PError> {
                let category = (msg_type & 0xF0) as u8;
                let type_id  = (msg_type & 0x0F) as u8;

                match category {
                    $(
                        $category => {
                            Ok(Message::$variant(<$type>::from_wire(type_id, payload)?))
                        }
                    )*
                    _ => Err(P2PError::UnknownMessageCategory(category)),
                }
            }
        }
    };
}