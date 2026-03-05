use tokio::fs::File;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut f = File::open("foo.txt").await?;
    
    // --- read() ---

    let mut buffer = [0; 10];
    println!("buffer = {:?}", buffer);

    // read up to 10 bytes
    let mut n = f.read(&mut buffer[..]).await?;

    println!("The bytes: {:?}", &buffer[..n]);

    n = f.read(&mut buffer[..]).await?;
    println!("The bytes: {:?}, size: {}", &buffer[..n], n);

    // --- read_to_end() ---
    
    f = File::open("foo.txt").await?;
    let mut buffer = Vec::new();

    // read the whole file
    let c = 15;
    n = f.read_to_end(&mut buffer).await?;
    println!("[read_to_end] size: {}, first {} bytes: {:?}", n, c, &buffer[..c]);

    // --- write() ---
    
    f = File::create("bar.txt").await?;

    // writes some prefix of the byte string, but not necessarily all of it.
    n = f.write(b"some bytes").await?;
    println!("[write] Wrote the first {} bytes of 'some bytes'.", n);

    // --- write_all() ---
    
    f = File::create("bar.txt").await?;
    f.write_all(b"some bytes").await?;
    println!("[write_all] wrote all bytes of 'some bytes'.");

    // --- copy() ---
    
    let mut reader: &[u8] = b"hello";
    let mut file = File::create("bar.txt").await?;

    io::copy(&mut reader, &mut file).await?;

    Ok(())
}
