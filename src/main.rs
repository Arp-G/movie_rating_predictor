use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};

// user id | item id | rating | timestamp
// #[derive(Debug)]
// struct Rating {
//     user_id: u32,
//     item_id: u32,
//     rating: u8,
// }

fn main() {
    //println!("{:?}", load_user_ratings("data/u.data").get(&1).unwrap());
    // load_movies("data/u.item");
    let mut user_ratings = load_user_ratings("data/u.data");

    let mut normalized_ratings = normalize_ratings(user_ratings);

    // Centered rating, sum almost 0
    // for rating in normalized_ratings.values() { 
    //     println!("{:?}", rating.values().sum::<f64>());
    // }
    
}

fn normalize_ratings(
    user_ratings: HashMap<u32, HashMap<u32, u8>>,
) -> HashMap<u32, HashMap<u32, f64>> {
    user_ratings
        .into_iter()
        .map(|(user_id, ratings)| {
              
            let total = ratings.values().map(|val| *val as u64).sum::<u64>();

            let avg_rating = total as f64 / ratings.values().count() as f64;

            let normalized_ratings = ratings
                .into_iter()
                .map(|(movie_id, rating)| (movie_id, rating as f64 - avg_rating))
                .collect();

            (user_id, normalized_ratings)
        })
        .collect()
}

fn load_moveis(path: &str) -> HashMap<u32, String> {
    let file = File::open(path).unwrap();
    BufReader::new(file)
        .lines()
        .map(|line| {
            let res = line.unwrap();
            let mut iter = res.split("|");
            // println!("{}, {}", iter.next().unwrap(), iter.next().unwrap());
            let movie_id = iter
                .next()
                .expect("Expected a movie id got none")
                .parse::<u32>()
                .unwrap();

            let movie_title = iter
                .next()
                .expect("Expected a movie title got none")
                .to_string();

            (movie_id, movie_title)
        })
        .collect()
}

fn load_user_ratings(path: &str) -> HashMap<u32, HashMap<u32, u8>> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    let mut users_hash = HashMap::new();
    for line in reader.lines() {
        let res = line.unwrap();
        let mut iter = res.split_whitespace();

        let user_id = iter
            .next()
            .expect("Expected a user id got none")
            .parse::<u32>()
            .unwrap();

        let movie_id = iter
            .next()
            .expect("Expected a item id got none")
            .parse::<u32>()
            .unwrap();

        let rating = iter
            .next()
            .expect("Expected a rating got none")
            .parse::<u8>()
            .unwrap();

        users_hash
            .entry(user_id)
            .or_insert(HashMap::new())
            .insert(movie_id, rating);
    }

    users_hash
}
