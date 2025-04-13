#[macro_export]
macro_rules! select_cancel {
    ( $future:expr , $cancellation_token:ident ) => {
        {
            let join_handle = tokio::spawn(async move {
                    select! {
                        _ = $future => {}
                        _ = $cancellation_token.cancelled() => {
                            info!("Cancelled by cancellation token.")
                        }
                    }
            });

            join_handle
        }
    };
}