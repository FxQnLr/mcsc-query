use mcsc_query::{basic_stats, full_stats, Error, FullStat, BasicStat};

const SERVER: &str = "localhost:25565";
const HOSTIP: &str = "192.168.178.28";

#[test]
fn basic() -> Result<(), Error> {
    let stat = BasicStat {
        motd: "A Minecraft Server".to_string(),
        gametype: "SMP".to_string(),
        map: "world".to_string(),
		numplayers: 0,
		maxplayers: 20,
		hostport: 25565,
		hostip: HOSTIP.to_string()
    };
    let res = basic_stats(SERVER)?;
    assert_eq!(res, stat);
    Ok(())
}

#[test]
fn full() -> Result<(), Error> {
    let stat = FullStat {
        hostname: "A Minecraft Server".to_string(),
        game_type: "SMP".to_string(),
        game_id: "MINECRAFT".to_string(),
        version: "1.20.4".to_string(),
        plugins: "".to_string(),
        map: "world".to_string(),
        numplayers: 0,
        maxplayers: 20,
        hostport: 25565,
		hostip: HOSTIP.to_string(),
        players: vec![],
    };
    let res = full_stats(SERVER)?;
    assert_eq!(res, stat);
    Ok(())
}
