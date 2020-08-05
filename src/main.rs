mod codec;

use crate::codec::{Message, OpcCodec};
use async_std::{
    net::{TcpListener, TcpStream},
    prelude::*,
    task,
};
use futures::TryStreamExt;
use futures_codec::Framed;
use rpi_led_matrix::{LedColor, LedMatrix, LedMatrixOptions};

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let mut options = LedMatrixOptions::new();
    options.set_rows(64);
    options.set_cols(64);
    let matrix = LedMatrix::new(Some(options)).map_err(|msg| anyhow::anyhow!("{}", msg))?;
    task::block_on(setup_server(matrix))
}

async fn setup_server(matrix: LedMatrix) -> anyhow::Result<()> {
    let listener = TcpListener::bind(("0.0.0.0", 7890)).await?;
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        connection_loop(stream, &matrix).await?;
    }
    Ok(())
}

async fn connection_loop(stream: TcpStream, matrix: &LedMatrix) -> anyhow::Result<()> {
    let mut framed = Framed::new(stream, OpcCodec);
    while let Some(frame) = framed.try_next().await? {
        match frame {
            Message::SetColors(_, colors) => {
                let mut canvas = matrix.canvas();
                let (width, height) = canvas.size();
                let mut x = 0;
                let mut y = 0;
                for (red, green, blue) in colors {
                    canvas.set(x, y, &LedColor { red, green, blue });
                    x += 1;
                    if x == width {
                        x = 0;
                        y += 1;
                    }
                    if y == height {
                        unreachable!("we've exceeded the canvas size");
                    }
                }
            }
        }
    }
    Ok(())
}
