//! What the parser actually gets out of the checked-in corpora.
//!
//! `cargo test` proves the parser does not panic; nothing proved it extracts anything. Run with
//! `cargo run --example coverage` before and after a change and diff the two.

fn main() {
    for (label, body) in [
        ("nyaa", include_str!("../tests/nyaa.txt")),
        ("anime", include_str!("../tests/anime.txt")),
    ] {
        let names: Vec<&str> = body.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
        let (mut episode, mut lone_episode, mut season, mut batch, mut title, mut resolution) = (0, 0, 0, 0, 0, 0);
        for name in &names {
            let Ok(r) = sacha::parse(name) else { continue };
            let eps = r.episode_terms.as_deref().unwrap_or(&[]);
            if !eps.is_empty() { episode += 1 }
            // a term usable as a NUMBER, which is what a consumer keys episodes by
            if eps.iter().any(|t| t.trim().parse::<u32>().is_ok()) { lone_episode += 1 }
            if r.season_terms.as_deref().is_some_and(|t| !t.is_empty()) { season += 1 }
            if r.batch_terms.as_deref().is_some_and(|t| !t.is_empty()) { batch += 1 }
            if r.titles.as_deref().is_some_and(|t| !t.is_empty()) { title += 1 }
            if r.resolution_terms.as_deref().is_some_and(|t| !t.is_empty()) { resolution += 1 }
        }
        let pct = |n: usize| format!("{:>5} ({:>4.1}%)", n, 100.0 * n as f64 / names.len() as f64);
        println!("{label}: {} names", names.len());
        println!("  any episode term  {}", pct(episode));
        println!("  a LONE number     {}", pct(lone_episode));
        println!("  season            {}", pct(season));
        println!("  batch             {}", pct(batch));
        println!("  title             {}", pct(title));
        println!("  resolution        {}", pct(resolution));
    }
}
