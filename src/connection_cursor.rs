/**
 * This is a reimplementation of `Connection` from `src/connection.rs`
 * which is more `manual` (i.e. uses `read()` and manually managed cursor,
 * instead of `bytes::BytesMut` which implements the `BufMut` trait).
 */

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use mini_redis::{Frame, Result};

pub struct Connection {
    stream: TcpStream,
    buffer: Vec<u8>,
    cursor: usize,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            // allocate the buffer with 4kb of capacity.
            buffer: vec![0; 4096],
            cursor: 0,
        }
    }

    /// Read a frame from the connection.
    ///
    /// Returns `None` if EOF is reached
    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            // attempt to parse a frame from the buffered data.
            // if enough data has been buffered, the frame is returned.
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            // ensure the buffer has capacity
            if self.buffer.len() == self.cursor {
                // grow the buffer
                self.buffer.resize(self.cursor * 2, 0);
            }

            // read into the buffer, tracking the number
            // of bytes read
            let n = self.stream.read(
                &mut self.buffer[self.cursor..]).await?;

            // there is not enough buffered data to read a frame.
            // attempt to read more data from the socket.
            //
            // on success, the number of bytes is returned.
            // `0` indicates "end of stream".
            if 0 == n {
                // the remote closed the connection.
                // for this to  be a clean shutdown, there should be no
                // data in the read buffer.
                // if there is, this means that the peer closed the socket
                // while sending a frame.
                if self.cursor == 0 {
                    return Ok(None);
                }
                return Err("connection reset by peer".into());
            }

            // update cursor
            self.cursor += n;
        }
    }

    /// Write a frame to the connection.
    pub async fn write_frame(&mut self, frame: &Frame) -> Result<()> {
        // implementation here
    }
}
