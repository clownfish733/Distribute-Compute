#[macro_export]
macro_rules! wire_message {
    ($enum_name:ident {
        $($variant:ident($type:ty) =$id:expr), * $(,)?
    }) => {
        impl $enum_name{
            fn to_wire(&self) -> Result<(u8, Vec<u8>), crate::core::P2PError>
            where 
                $($type: crate::core::traits::WireEncodable, )*
            {
                match self {
                    $(
                        $enum_name::$variant(msg) => Ok(($id, msg.encode()?)),
                    )*
                }
            }

            fn from_wire(type_id: u8, payload: &[u8]) -> Result<Self, crate::core::P2PError>
            where 
                $($type: crate::core::traits::WireEncodable, )*
            {
                match type_id{
                    $(
                        $id => Ok($enum_name::$variant(<$type>::decode(payload)?)),
                    )*
                    _ => Err(crate::core::P2PError::UnknownMessageType(type_id))
                }
            }
        }
    }
}

#[macro_export]
macro_rules! message_dispatch {
    ($($variant:ident($type:ty) = $category:expr), * $(,)?) => {
        impl Message{
            pub fn to_bytes(&self) -> Result<Vec<u8>, crate::core::P2PError> {
                let (category, type_id, payload) = match self {
                    $(
                        Message::$variant(msg) => {
                            let (id, payload) = msg.to_wire()?;
                            ($category, id, payload)
                        }
                    )*
                };

                let msg_type = category + type_id;
                let total_len = 1 + payload.len();
                let mut buf = Vec::with_capacity(4 + total_len);
                buf.extend_from_slice(&(total_len as u32).to_be_bytes());
                buf.push(msg_type);
                buf.extend_from_slice(&payload);
                Ok(buf)
            }

            pub async fn from_stream<R: tokio::io::AsyncReadExt + Unpin>(
                stream: &mut R,
            ) -> Result<Self, crate::core::P2PError> {
                let mut len_buf = [0u8; 4];
                stream.read_exact(&mut len_buf).await?;
                let msg_len = u32::from_be_bytes(len_buf) as usize;

                if msg_len == 0 || msg_len > crate::core::constants::MAX_MESSAGE_SIZE {
                    return Err(crate::core::P2PError::InvalidMessageSize(msg_len))
                }

                let msg_type = stream.read_u8().await?;
                let payload_len = msg_len - 1;
                let mut payload = vec![0u8; payload_len];
                stream.read_exact(&mut payload).await?;

                let category = msg_type & 0xF0;
                let type_id = msg_type & 0x0F;

                match category{
                    $(
                        $category => Ok(Message::$variant(<$type>::from_wire(type_id, &payload)?)),
                    )*
                    _ => Err(crate::core::P2PError::UnknownMessageCategory(category))
                }
        }
        }

        
    }
}