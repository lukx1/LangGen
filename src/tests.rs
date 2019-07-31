use super::*;

#[test]
#[warn(unreachable_patterns)]
fn test_acc() -> Result<(),Box<Error>>{
    let mut lang = MyLang::new_with_config(Config::default());
    let mut gen = RandomEngine::new(lang,Box::new(CalculatedRandom::new()));

    println!("Syllables: {:?}\n\n",gen.my_lang.syllables);

    {
        fs::write("LangGenCfg/Word_Database.txt", "".as_bytes())?;
    }

    for i in 0..200{

        gen.my_lang.recalc_real();
        gen.recalc_adjusted_random();
        let words= gen.create_words(2,6,10);
        for word in &words {
            gen.my_lang.add_to_db(word)?;
        }

        let mut offby = 0f64;

        for syllable in &gen.my_lang.syllables {
            let real = gen.my_lang.occ_real.get(syllable).unwrap();
            let wanted = gen.my_lang.occ_wanted.get(syllable).unwrap();

            if wanted > real {
                offby += (1.0 - wanted/real).abs();
            }
            else {
                offby += (1.0 - real/wanted).abs();
            }
        }

        println!("Off by {:.1}",offby);
    }

    println!("\n---\n");

    println!("syllable;wanted;real");

    for syllable in &gen.my_lang.syllables {
        let real = gen.my_lang.occ_real.get(syllable).unwrap();
        let wanted = gen.my_lang.occ_wanted.get(syllable).unwrap();
        println!("{};{};{}",syllable,wanted,real);
    }

    Ok(())
}
