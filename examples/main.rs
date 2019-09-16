fn main() {
    futures::executor::block_on(async {
        async fn main() -> Result<(), std::io::Error> {
            use async_macros::try_select;
            use futures::future;
            use std::io::{Error, ErrorKind};

            let a = future::pending::<Result<u8, Error>>();
            let b = future::ready(Err(Error::from(ErrorKind::Other)));
            let c = future::ready(Ok(1u8));

            assert_eq!(try_select!(a, b, c).await?, 1u8);

            use async_macros::JoinStream;
            use futures::stream::{self, StreamExt};
            use futures::future::ready;

            Ok(())
        }
        main().await.unwrap();
    });
}
