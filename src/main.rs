use cliclack as cc;
use ndm::Dice;

const ATTACK: i32 = 23;
fn main() -> anyhow::Result<()> {
    ctrlc::set_handler(|| {})?;
    cc::intro("dmg-roll")?;
    let attack_type = cc::select("Attack type?")
        .item("fist", "Powerful Fist", "+23/19/15")
        .item("laser", "Laser Beam", "+23/19/15")
        .interact()?;
    let buffed = cc::confirm("Inner Upheavel?").interact()?;
    let fob = cc::confirm("Flurry of Blows?")
        .initial_value(buffed)
        .interact()?;
    let map = cc::select("Multiple attack penalty?")
        .item(0, "-0", "")
        .item(-4, "-4", "")
        .item(-8, "-8", "")
        .interact()?;
    let mut hits = vec![roll("1d20") + ATTACK + map + if buffed { 1 } else { 0 }];
    if fob {
        hits.push(roll("1d20") + ATTACK + map - 4 + if buffed { 1 } else { 0 })
    }
    let dmg_die = cc::select("Damage die?")
        .item("1d6", "1d6", "Standard")
        .item("1d8", "1d8", "3-4 instances of Off-Guard")
        .item("10", "1d10", "5+ instances of Off-Guard")
        .interact()?;
    let mut results = vec![];
    for hit in hits {
        results.push(
            cc::select(format!("Does a {} hit?", hit))
                .item(Some(false), "Hit", "")
                .item(None, "Miss", "").initial_value(None)
                .item(Some(true), "Crit", "")
                .interact()?,
        );
    }
    // handle the attacks
    for crit in results.iter().cloned().filter_map(|x| x) {
        let mut dmg = vec![
            ("", vec![dmg_die; 3]),
            (" fire", vec![dmg_die]),
            (" electricity", vec![dmg_die]),
        ];
        match attack_type {
            "laser" => dmg[1].1.push("2"), // 2 fire
            "fist" => dmg[0].1.push("6"),  // 6 normal
            _ => {}
        }
        if buffed {
            // add 2d6 normal
            dmg[0].1.extend_from_slice(&[dmg_die, dmg_die])
        }
        let (short, long, all) = dmg
            .into_iter()
            .map(|(ty, dice)| {
                (
                    ty,
                    dice.into_iter()
                        .map(roll)
                        .map(|x| if crit { x * 2 } else { x }),
                )
            })
            .map(|(ty, mut dice)| {
                let sum: i32 = dice.clone().sum(); // todo uh
                let first = dice.next().unwrap().to_string();
                (
                    dice.fold(first, |acc, x| acc + " + " + &x.to_string()) + ty,
                    sum,
                    ty,
                )
            })
            .inspect(|x| println!("{:?}", x))
            .fold(
                ("".to_string(), "".to_string(), 0),
                |(short, long, all), (desc, sum, ty)| {
                    (
                        short + " + " + &sum.to_string() + ty,
                        long + "\n" + &desc.to_string(),
                        all + sum,
                    )
                },
            );
        cc::note(
            format!(
                "{}{} = {}",
                if crit { "crit! " } else { "" },
                &short[3..], // ignore leading ' + '
                all
            ),
            &long[1..], // ignore leading \n
        )?;
        if crit {
            cc::log::info("crit! Deal 1d10 persistent fire damage")?;
            cc::log::info("crit! Arc electric damage to 2 nearby targets")?;
            if !cc::confirm("crit! Did target succeed a fortitude save vs. your class DC?")
                .initial_value(true)
                .interact()?
            {
                cc::log::info(format!("crit! Target is {} 1 until the end of your next turn.", match attack_type {
                    "laser" => "dazzled",
                    "fist" => "slowed",
                    _ => "huh whuh"
                }))?;
            };
        }
    }
    if results.len() != 0 && fob {
        let stun = cc::select("Did target succeed a fortitude save vs. your class DC? (select \"yes\" if multiple targets)")
            .item(0, "Success+", "no effect")
            .item(1, "Failure", "stunned 1")
            .item(3, "Critical Failure", "stunned 3")
            .interact()?;
        if stun != 0 {
            cc::log::info(format!("Target is stunned {stun} until the end of your next turn."))?;
        }
    }

    Ok(())
}

fn roll(x: &str) -> i32 {
    x.parse::<Dice>()
        .map(|x| x.total())
        .or(x.parse::<i32>())
        .unwrap()
}
