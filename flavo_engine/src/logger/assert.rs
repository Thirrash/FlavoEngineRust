
pub fn assert_error<Pred, Msg>(expression: Pred, msg_func: Msg)
where Pred: Fn() -> bool, Msg: Fn() -> String
{
    if cfg!(debug_assertions) && !expression() {
        let msg: String = msg_func();
        eprintln!("{}", msg);
    }
}
