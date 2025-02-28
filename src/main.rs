use cliclack as cc;
use ndm::Dice;

const ATTACK: i32 = 23;
fn main() -> anyhow::Result<()> {
    let mut total_dmg = 0;
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
    let d1: i32 = roll("1d20");
    let mut hits = vec![(d1 + ATTACK + map + if buffed { 1 } else { 0 }, d1)];
    if fob {
        let d2: i32 = roll("1d20");
        hits.push((d2 + ATTACK + map - 4 + if buffed { 1 } else { 0 }, d2))
    }
    let dmg_die = cc::select("Damage die?")
        .item("1d6", "1d6", "Standard")
        .item("1d8", "1d8", "3-4 instances of Off-Guard")
        .item("10", "1d10", "5+ instances of Off-Guard")
        .interact()?;
    let mut results = vec![];
    for (hit, roll) in hits {
        results.push(
            cc::select(format!("Does a {} hit? (rolled: {})", hit, roll))
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
            (" mental", vec![dmg_die]),
        ];
        match attack_type {
            // .into_iter()
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
                        .map(|x| if crit { x * 2 } else { x })
                        .collect::<Vec<_>>() // roll them NOW to collapse the states
                        .into_iter()
                )
            })
            // .inspect(|x| println!("{:?}", x))
            .map(|(ty, mut dice)| {
                // dbg!(&dice.clone().collect::<Vec<_>>());
                let sum: i32 = dice.clone().sum(); // todo uh (what did she mean by this???)
                total_dmg += sum;
                let first = dice.next().unwrap().to_string();
                // println!("first: {first}");
                (
                    dice
                        // .inspect(|x| println!("{:?}", x))
                        .fold(first, |acc, x| acc + " + " + &x.to_string()) + ty,
                    sum,
                    ty,
                )
            })
            // .inspect(|x| println!("{:?}", x))
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
            cc::log::info("crit! Deal 2d10 persistent fire damage")?;
            cc::log::info("crit! Target is stupefied 1, and frightened 2 if they were already stupified.")?;
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
    let _ = cc::outro(format!("total dmg: {}", total_dmg));

    Ok(())
}

fn roll(x: &str) -> i32 {
    x.parse::<Dice>()
        .map(|x| x.total())
        .or(x.parse::<i32>())
        .unwrap()
}
