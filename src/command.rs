use anyhow::Result;
use crate::connection::Connection;
use crate::db::Db;
use crate::frame::Frame;
use crate::shutdown::Shutdown;

pub enum Command {
    Get()
}


impl Command {
    pub fn from(frame:Frame) -> Result<Command> {
        Ok(Command::Get())
    }

    pub async fn apply(self,db:&Db,dst:&mut Connection,shutdown:&mut Shutdown) -> Result<()> {
        Ok(())
    }
}