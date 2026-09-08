use crate::{Error, Result, Transport};
use std::time::Duration;
#[derive(Clone,Copy,Debug,Eq,PartialEq)] pub enum Command { PersistentWrite=0xC0, Read=0xC1, TemporaryWrite=0xC2 }
#[derive(Clone,Debug)] pub struct RawTransaction {pub request:Vec<u8>,pub response:Vec<u8>}
pub(crate) fn transact(t:&mut dyn Transport,cmd:Command,start:u8,data:&[u8],read_len:usize,timeout:Duration)->Result<(Vec<u8>,RawTransaction)> {
    let len=if matches!(cmd,Command::Read){read_len}else{data.len()};
    if len==0||len>255{return Err(Error::InvalidRegisterRange{start,length:len});}
    let mut req=vec![cmd as u8,start,len as u8]; if !matches!(cmd,Command::Read){req.extend_from_slice(data)}
    t.write_all(&req,timeout)?;
    let mut hdr=[0u8;3];t.read_exact(&mut hdr,timeout)?;
    if hdr==[0xff;3]{return Err(Error::ModuleFormatError)}
    if hdr[0]!=0xc1{return Err(Error::UnexpectedResponseCommand(hdr[0]))}
    if hdr[1]!=start{return Err(Error::UnexpectedResponseAddress{expected:start,actual:hdr[1]})}
    if hdr[2] as usize!=len{return Err(Error::UnexpectedResponseLength{expected:len as u8,actual:hdr[2]})}
    let mut body=vec![0;len];t.read_exact(&mut body,timeout)?;
    let mut response=hdr.to_vec();response.extend_from_slice(&body);
    if !matches!(cmd,Command::Read)&&body!=data{return Err(Error::WriteEchoMismatch{expected:data.to_vec(),actual:body})}
    Ok((body,RawTransaction{request:req,response}))
}
