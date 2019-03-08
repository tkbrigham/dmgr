pub fn parse_args(s: Vec<String>) -> Result<Subcommand, String> {
    match s[1].as_str() {
        "config" => Ok(Subcommand::Config),
        "health" | "h" => Ok(Subcommand::Health),
        "list" | "ls" => Ok(Subcommand::List),
        "logs" => Ok(Subcommand::Logs),
        "pull" => Ok(Subcommand::Pull),
        "register" | "r" => Ok(Subcommand::Register),
        "restart" => Ok(Subcommand::Restart),
        "start" => Ok(Subcommand::Start),
        "stop" => Ok(Subcommand::Stop),
        invalid @ _ => Err(format!("not a valid subcommand: {}", invalid))
    }
}

#[derive(Debug)]
pub enum Subcommand {
    Config,
    Health,
    List,
    Logs,
    Pull,
    Register,
    Restart,
    Start,
    Stop,
}
