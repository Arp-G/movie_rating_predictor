use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};
extern crate colored;
use colored::*;

const K: usize = 10; // PICK "K" most similar users for prediction

fn main() {
    run(276);
}

fn run(user_id: u32) {
    let movies = load_movies("data/u.item");
    let ratings = load_user_ratings("data/u.data");
    let normalized_ratings = normalize_ratings(&ratings);

    let mut total = 0;
    let mut green = 0;
    let mut yellow = 0;
    let mut red = 0;
    for (movie_id, actual_rating) in ratings.get(&user_id).unwrap().iter() {
        let predicted_rating =
            predict_rating(user_id, *movie_id, &ratings, &normalized_ratings, &movies);

        let diff = (*actual_rating as f64 - predicted_rating).abs();

        total += 1;

        let predicted_rating = if diff <= 0.5 {
            green += 1;
            predicted_rating.to_string().green()
        } else if diff <= 1.0 {
            yellow += 1;
            predicted_rating.to_string().yellow()
        } else {
            red += 1;
            predicted_rating.to_string().red()
        };

        println!(
            "Actual Rating: {}, Predicted Rating: {}",
            actual_rating,
            predicted_rating.bold()
        );
    }

    println!("--------------------------------------");
    println!("Total: {}", total);
    println!("Correct Predictions: {}", green.to_string().green().bold());
    println!(
        "Somewhat correct Predictions: {}",
        yellow.to_string().yellow().bold()
    );
    println!("Wrong Predictions: {}", red.to_string().red().bold());
}

fn predict_rating(
    user_id: u32,
    movie_id: u32,
    ratings: &HashMap<u32, HashMap<u32, u8>>,
    normalized_ratings: &HashMap<u32, HashMap<u32, f64>>,
    movies: &HashMap<u32, String>,
) -> f64 {
    let similar_users =
        find_k_most_similar_users(K, user_id, movie_id, normalized_ratings, movies);

    calculate_rating(similar_users, movie_id, ratings)
}

fn calculate_rating(
    similar_users: Vec<(u32, f64)>,
    target_movie: u32,
    ratings: &HashMap<u32, HashMap<u32, u8>>,
) -> f64 {
    let (numerator, denominator) = similar_users.iter().fold(
        (0.0, 0.0),
        |(numerator, denominator), (user_id, similarity)| {
            let rating = ratings.get(user_id).unwrap().get(&target_movie).unwrap();
            (
                numerator + *rating as f64 * similarity,
                denominator + similarity,
            )
        },
    );

    numerator / denominator
}

fn find_k_most_similar_users(
    k: usize,
    target_user: u32,
    target_movie: u32,
    ratings: &HashMap<u32, HashMap<u32, f64>>,
    movies: &HashMap<u32, String>,
) -> Vec<(u32, f64)> {
    let target_user_ratings = ratings.get(&target_user).unwrap();
    // find users who have rated <target_movie>
    let mut similar_users: Vec<(u32, f64)> = ratings
        .clone()
        .into_iter()
        .filter(|(user_id, user_ratings)| {
            user_ratings.contains_key(&target_movie) && *user_id != target_user
        })
        .map(|(user_id, user_ratings)| {
            let similarity = cosine_similarity(&movies, target_user_ratings, &user_ratings);
            (user_id, similarity)
        })
        .collect::<Vec<(u32, f64)>>();

    // Sort to get similar users
    similar_users.sort_unstable_by(|(_, sim1), (_, sim2)| sim2.partial_cmp(sim1).unwrap());

    // Pick k most similar users
    similar_users.into_iter().take(k).collect()
}

// https://neo4j.com/docs/graph-algorithms/current/labs-algorithms/cosine/#:~:text=Cosine%20similarity%20is%20the%20cosine,'%20lengths%20(or%20magnitudes).
// cos(θ) = A.B / (|A|.|B|)
fn cosine_similarity(
    movies: &HashMap<u32, String>,
    user1_ratings: &HashMap<u32, f64>,
    user2_ratings: &HashMap<u32, f64>,
) -> f64 {
    // A.B
    let numerator = movies.keys().fold(0.0, |acc, movie_id| {
        let user1_rating = user1_ratings.get(movie_id).unwrap_or(&0.0);
        let user2_rating = user2_ratings.get(movie_id).unwrap_or(&0.0);
        acc + (user1_rating * user2_rating)
    });

    // |A|.|B|
    let denominator = magnitude(user1_ratings) * magnitude(user2_ratings);

    // A.B / (|A|.|B|)
    numerator / denominator
}

fn magnitude(rating_vec: &HashMap<u32, f64>) -> f64 {
    rating_vec
        .values()
        .fold(0.0, |acc, rating| acc + rating * rating)
        .sqrt()
}

fn normalize_ratings(ratings: &HashMap<u32, HashMap<u32, u8>>) -> HashMap<u32, HashMap<u32, f64>> {
    ratings
        .clone()
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

fn load_movies(path: &str) -> HashMap<u32, String> {
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
