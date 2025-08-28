/*
가위바위보 아이디 -> 3비트
신청후 거절 전까지 신청 불가
수락 비 수락  3분 동안 안하면 자동 비수락 처리
1분안에 안고르면 패배 처리
지면 끄는 권리 가짐
 */
extern crate core;

use bevy_tasks::IoTaskPool;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::io;
use std::io::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;

type CopyStr = Arc<String>;

#[derive(Clone,Debug)]
pub struct Data {
    pub token: Option<CopyStr>,
    pub type_kind: DataTypeKind,
    pub inform: DataType,
}

impl Data {

    pub async fn read_data<T>(socket:&mut T) -> Result<Data,Error>
    where
        T: AsyncRead + Unpin
    {
        let kind : DataTypeKind;
        let inform: DataType;
        let token: Option<CopyStr>;

        let mut data_type = [0; 1];
        socket.read_exact(&mut data_type).await?;
        let type_kind= DataTypeKind::from_u8(data_type[0]);
        if let Some(_kind) = type_kind{
            kind = _kind;
        }else { return Err(Error::new(io::ErrorKind::InvalidData,"Type Error"));}

        let mut token_len = [0; 4];
        socket.read_exact(&mut token_len).await?;
        let len = u32::from_be_bytes(token_len);

        let mut token_inform = vec![0; len as usize];
        socket.read_exact(&mut token_inform).await?;

        let cow_token = String::from_utf8_lossy(&token_inform);
        let token_string = cow_token.trim().to_string();

        token = if token_string == "None" { None } else { Some(Arc::new(token_string)) };

        let mut data_len = [0; 4];
        socket.read_exact(&mut data_len).await?;

        let len = u32::from_be_bytes(data_len);
        let mut data_inform = vec![0; len as usize];
        socket.read_exact(&mut data_inform).await?;

        inform = DataType::from_bytes(kind,&data_inform);

        Ok(Data {
            token: token,
            type_kind: kind,
            inform: inform
        })
    }

    pub async fn write_data<T>(socket:&mut T, data: Data) -> Result<(),Error>
    where
        T: AsyncWrite + Unpin
    {
        let type_num = data.type_kind as u8;
        let send_data = data.inform.change_bytes();
        let len : [u8; 4] = (send_data.len() as u32).to_be_bytes();

        let token = if let Some(token) = data.token { token }else { Arc::new("None".to_string()) };
        let token_len: [u8; 4] = (token.len() as u32).to_be_bytes();

        let mut buffer = Vec::new();
        buffer.push(type_num);
        buffer.extend_from_slice(&token_len);
        buffer.extend_from_slice(token.as_bytes());
        buffer.extend_from_slice(&len);
        buffer.extend_from_slice(&*send_data);

        socket.write_all(&buffer).await?;
        socket.flush().await?;

        sleep(Duration::from_millis(12)).await;

        Ok(())
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum  DataTypeKind {
    Token = 001,
    Name = 002,
    Remove = 003,
    Message = 101,
    Image = 102,
    RPS = 103, // 가위 바위 보
}

impl DataTypeKind{

    fn from_u8(num: u8) -> Option<Self>{
        match num {
            001 => Some(Self::Token),
            002 => Some(Self::Name),
            003 => Some(Self::Remove),
            101 => Some(Self::Message),
            102 => Some(Self::Image),
            103 => Some(Self::RPS),
            _ => None
        }
    }
}

#[derive(Clone,Debug)]
pub enum DataType {
    Token(CopyStr),
    Name(CopyStr),
    Remove,
    Message(CopyStr),
    Image,
    RPS(RPSType)
}

impl DataType {

    fn from_bytes(kind: DataTypeKind,bytes: &[u8]) -> Self{
        match kind {
            DataTypeKind::Token => {
                let token = String::from_utf8_lossy(&bytes);
                let token_string = token.trim().to_string();
                DataType::Token(Arc::new(token_string))
            }
            DataTypeKind::Name => {
                let name = String::from_utf8_lossy(&bytes);
                let name_string = name.trim().to_string();
                DataType::Name(Arc::new(name_string))
            }
            DataTypeKind::Remove => {
                DataType::Remove
            }
            DataTypeKind::Message => {
                let msg = String::from_utf8_lossy(&bytes);
                let msg_string = msg.trim().to_string();
                DataType::Message(Arc::new(msg_string))
            }
            DataTypeKind::Image => {
                DataType::Image
            }
            DataTypeKind::RPS => {
                let rps_type = bytes[0].clone();
                let id = bytes[1..4].to_vec();
                match rps_type {
                    1 => {
                        let token = String::from_utf8_lossy(&bytes[4..]);
                        let token_string = token.trim().to_string();
                        DataType::RPS(RPSType::Send(id,Arc::new(token_string)))
                    }
                    2 => {
                        DataType::RPS(RPSType::Accept(id,bytes[1] == 1))
                    }
                    3 => {DataType::RPS(RPSType::Rock(id))}
                    4 => {DataType::RPS(RPSType::Paper(id))}
                    5 => {DataType::RPS(RPSType::Scissor(id))}
                    _ => {DataType::RPS(RPSType::Rock(vec![0]))}
                }
            }
        }
    }

    pub fn change_bytes(&self) -> Vec<u8>{
        match self {
            DataType::Token(info) |
            DataType::Name(info) |
            DataType::Message(info) => {
                info.as_bytes().to_vec()
            },
            DataType::RPS(rps) => {
                match rps {
                    RPSType::Send(id,token) => {
                        [[1].as_slice(),id.as_slice(),token.as_bytes()].concat()
                    }
                    RPSType::Accept(id,is_accept) => {
                        let index_1 = if is_accept.clone() {1}else { 0 };
                        [[2,index_1].as_slice(),id.as_slice()].concat()
                    }
                    RPSType::Rock(id) => {
                        [[3].as_slice(),id.as_slice()].concat()
                    }
                    RPSType::Paper(id) => {
                        [[4].as_slice(),id.as_slice()].concat()
                    }
                    RPSType::Scissor(id) => {
                        [[5].as_slice(),id.as_slice()].concat()
                    }
                }
            }
            _ => {vec![0]}
        }
    }
}

#[derive(Clone, Debug)]
pub enum RPSType {
    Send(Vec<u8>,CopyStr),
    Accept(Vec<u8>,bool),
    Rock(Vec<u8>),
    Paper(Vec<u8>),
    Scissor(Vec<u8>)
}

pub fn set_up_tokio() -> Runtime{
    let rt = Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt

}

pub fn tokio_spawn<F>(rt: Arc<Runtime>,future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    let pool =IoTaskPool::get();
    let rt = rt.clone();
    pool.spawn(async move{
        rt.spawn(future);
    }).detach()
}

pub async fn server_host(addr: String,off: Sender<bool>) -> Result<(), Error> {
    println!("Server Addr: {}",addr);
    let server = TcpListener::bind(addr.trim()).await?;
    let (tx, _) = broadcast::channel::<Data>(100);
    let _off = off.clone();
    let users = Arc::new(Mutex::new(HashMap::<String, String>::new()));
    let handle = tokio::spawn(async move {
        loop {
            if let Ok((socket, addr)) = server.accept().await {
                let tx = tx.clone();
                let rx = tx.subscribe();
                let off_tx = _off.clone();
                let off = off_tx.subscribe();
                let users = Arc::clone(&users);
                let r = socket.set_nodelay(true);
                println!("No delay (Server): {:?}",r);
                tokio::spawn(async move {
                    process_socket(socket,addr,tx,rx,users,off).await
                });
            }else {
                println!("Fail Host");
            }
        }
    });
    let tx = off.clone();
    let mut rx = tx.subscribe();

    'parent: loop {
        while let Ok(off) = rx.recv().await {
            if off {
                handle.abort();
                break 'parent;
            }
        }
    }

    println!("Server Out");
    Ok(())

}

pub async fn join_server(token: String, name: String, addr: String) -> Result<TcpStream, Error> {

    let mut socket = TcpStream::connect(addr.trim()).await?;
    let r = socket.set_nodelay(true);
    println!("No delay (Client): {:?}",r);

    Data::write_data(&mut socket,Data {
        token: None,
        type_kind: DataTypeKind::Token,
        inform: DataType::Token(Arc::new(token.clone()))
    }).await?;

    Data::write_data(&mut socket,Data {
        token: None,
        type_kind: DataTypeKind::Name,
        inform: DataType::Name(Arc::new(name.clone()))
    }).await?;

    Ok(socket)

}

async fn process_socket(mut socket: TcpStream, addr: SocketAddr, tx: Sender<Data>, mut rx: Receiver<Data>, users: Arc<Mutex<HashMap<String, String>>>, mut off:Receiver<bool>) {
    let mut token: String = String::new();
    println!("----- Server -----");
    println!("Addr: {:?}",addr);
    loop {
        let read = Data::read_data(&mut socket).await;
        if let Ok(read) = read {
            if read.type_kind == DataTypeKind::Token {
                if let DataType::Token(_token) = read.inform {
                    token = _token.to_string();
                }
                break
            }
        }
    }
    println!("Token: {:?}", token);

    let mut name = String::new();

    loop {
        let read = Data::read_data(&mut socket).await;
        if let Ok(read) = read {
            if read.type_kind == DataTypeKind::Name {
                if let DataType::Name(_name) = read.inform {
                    name = _name.to_string();
                }
                break
            }
        }
    }
    println!("Name: {:?}", name);

    let users_send_list = {
        let users_guard = users.lock().await;
        users_guard.clone()
    };

    for (token,name) in users_send_list.iter(){
        let _ = Data::write_data(&mut socket,Data {
            token: Some(Arc::new(token.clone())),
            type_kind: DataTypeKind::Name,
            inform: DataType::Name(Arc::new(name.clone()))
        }).await;
    }

    users.lock().await.insert(token.clone(),name.clone());
    println!("Users: {:?}",users);

    let _ = tx.send(Data {
        token: Some(Arc::new(token.clone())),
        type_kind: DataTypeKind::Name,
        inform: DataType::Name(Arc::new(name.clone()))
    });

    let (mut r_stream, mut w_stream) = socket.into_split();
    let _token = token.clone();

    let handle = tokio::spawn(async move {
        loop {
            match Data::read_data(&mut r_stream).await {
                Ok(data) => {
                    let _ = tx.send(data);
                }
                Err(_) => {
                    //println!("Server Err: {:?}",e);
                    let _ = tx.send(Data {
                        token: Some(Arc::new(_token.clone())),
                        type_kind: DataTypeKind::Remove,
                        inform: DataType::Remove
                    });
                    return;
                }
            }
        }
    });

    let handle_w = tokio::spawn(async move{
        loop {
            while let Ok(data) = rx.try_recv() {
                let rst = Data::write_data(&mut w_stream,data.clone()).await;
                println!("Write(Server): {:?}, {:?}",data,rst);
                if data.type_kind == DataTypeKind::Remove {
                    if let Some(_token) = data.token{
                        println!("{:?},{:?}",_token,token);
                        if _token.as_str() == token.clone().as_str(){
                            println!("End: {:?}",addr);
                            return;
                        }
                    }
                }
            }
        }
    });

    loop {
        while let Ok(off) = off.recv().await {
            if off {
                handle.abort();
                handle_w.abort();
                return;
            }
        }
    }
}

