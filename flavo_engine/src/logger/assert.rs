const ASSERT_ENABLE: bool = cfg!(debug_assertions);

pub fn assert_error<Pred, Msg>(expression: Pred, msg_func: Msg)
where Pred: Fn() -> bool, Msg: Fn() -> String
{
    if ASSERT_ENABLE && !expression() {
        let msg: String = msg_func();
        eprintln!("{}", msg);
    }
}
