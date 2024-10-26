use crate::{*, color::*};

fn bytes_to_readable_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&byte| {
            if byte.is_ascii_graphic() || byte == b' ' {
                (byte as char).to_string()
            } else {
                format!("{:02X}", byte)
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn bytes_to_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&byte| {
            format!("{:02X}", byte)
        })
        .collect::<Vec<String>>()
        .join(" ")
}

#[test]
fn test_to_string() {
    let component = TextComponent::from("This is a test!");
    assert_eq!(
        component.to_string(), 
        "{\"text\":\"This is a test!\"}".to_string()
    );
}

use std::io::{Cursor, Write};
use ferrumc_macros::{NetEncode, packet};
use ferrumc_net_codec::{
    encode::{NetEncode, NetEncodeOpts},
    decode::{NetDecode, NetDecodeOpts},
    net_types::var_int::VarInt
};
use ferrumc_nbt::NBTSerializeOptions;
use ferrumc_nbt::NBTSerializable;
use tokio::io::AsyncWriteExt;
use std::fs::File;

#[derive(NetEncode)]
#[packet(packet_id = 0x6C)]
struct TestPacket {
    message: TextComponent,
    overlay: bool,
}

#[tokio::test]
async fn test_serialize_to_nbt() {
    let component = TextComponentBuilder::new("test")
        .color(NamedColor::Blue)
        .build();
    //println!("{:#?}", component.color);
    println!("{}", component.to_string());
    println!("{}", bytes_to_readable_string(&component.serialize_nbt()[..]));

    println!("{}", component.serialize_nbt().len());

    //println!("\n{}", bytes_to_readable_string(&component.content.serialize_as_network()[..]));

    let mut file = File::create("foo.nbt").unwrap();
    /*let mut bytes = Vec::new();
    NBTSerializable::serialize(&vec![component.clone()], &mut bytes, &NBTSerializeOptions::Network);
    file.write_all(&bytes).unwrap();
    println!("\n{}\n", bytes_to_readable_string(&bytes[..]));*/
    file.write_all(&component.serialize_nbt()[..]).unwrap();

    let mut cursor = Cursor::new(Vec::new());
    TestPacket::encode_async(&TestPacket {
        message: TextComponentBuilder::new("test")
            .color(NamedColor::Blue)
            .build(),
        overlay: false,
    }, &mut cursor, &NetEncodeOpts::WithLength).await.unwrap();

    println!("\n{}\n", bytes_to_string(&cursor.get_ref()[..]));

    cursor.set_position(0);

    let length = VarInt::decode(&mut cursor, &NetDecodeOpts::None).unwrap();
    let id = VarInt::decode(&mut cursor, &NetDecodeOpts::None).unwrap();

    println!("{}\n", bytes_to_string(&component.serialize_nbt()[..]));

    println!("id: {}, length: {}, left: {}", id.val, length.val, length.val as u64 - cursor.position());
    println!("{}", bytes_to_readable_string(&cursor.get_ref()[cursor.position() as usize..]));
}
