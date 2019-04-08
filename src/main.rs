use std::fmt;
use rand::seq::SliceRandom;
use rand::Rng;
use std::time::{Duration, Instant};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct State {
	tiles: [u8; 16]
}

impl State{
	fn add_random_value(&mut self, blanks: Vec<usize>) {
		let mut rng = rand::thread_rng();
		let index : usize = *blanks.choose(&mut rng).unwrap();
		if 0.9 > rng.gen() {
			self.tiles[index] = 1u8;
		}
		else {
			self.tiles[index] = 2u8;
		}
	
	}

	fn shift(&mut self, start: i32, delta: i32) {
		let mut current_index = start;
		let mut current_target = start;
		let mut prev_val = 200u8;

		for _ in 0..4 {
			let val = self.tiles[current_index as usize];
			self.tiles[current_index as usize] = 0;
			if val == 0 {
				current_index += delta;
				continue;
			}
			else if val == prev_val {
				self.tiles[(current_target - delta) as usize] += 1u8;
				prev_val = 200u8;
			}
			else {
				self.tiles[current_target as usize] = val;
				current_target += delta;
				prev_val = val;
			}
			current_index += delta;
		}
	}

	fn shift_all(&mut self, start: i32, delta: i32, start_delta: i32) {
		for i in 0..4 {
			self.shift(start + i * start_delta, delta);
		}
	}

	fn blanks(&self) -> Vec<usize> {
		let mut result = Vec::with_capacity(16);
		for index in 0..16usize {
			if self.tiles[index] == 0u8 {
				result.push(index);
			}
		}
		return result;
	}

	fn next_states(&self) -> Vec<State> {
		let mut result = Vec::with_capacity(4);
		let mut shift_down = *self;
		let mut shift_up = *self;
		let mut shift_right = *self;
		let mut shift_left = *self;
		shift_down.shift_all(12, -4, 1);
		if shift_down != *self {
			result.push(shift_down)
		}
		shift_up.shift_all(0, 4, 1);
		if shift_up != *self {
			result.push(shift_up)
		}
		shift_right.shift_all(3, -1, 4);
		if shift_right != *self {
			result.push(shift_right);
		}
		shift_left.shift_all(0,1,4);
		if shift_left != *self {
			result.push(shift_left)
		}
		return result;
	}

	fn play_random_game(&mut self) {
		let mut next_states = self.next_states();
		let mut rng = rand::thread_rng();
		while next_states.len() > 0 {
			*self = *next_states.choose(&mut rng).unwrap();
			self.add_random_value(self.blanks());
			next_states = self.next_states();
		}
	}

	fn score(&self) -> i32 {
		let mut result = 0;
		for val in self.tiles.iter() {
			if (*val != 0u8) {
				result += 1 << val;
			}
		}
		return result;
	}

	fn average_score(&self, iterations: i32) -> i32 {
		let mut total_score = 0;
		for _ in 0..iterations {
			let mut new_game = *self;
			new_game.play_random_game();
			total_score += new_game.score();
		}
		return total_score / iterations;
	}

	fn average_score_time(&self, duration: u128) -> i32 {
		let start = Instant::now();
		let mut total_score = 0;
		let mut num_games = 0;
		while start.elapsed().as_millis() < duration {
			let mut new_game = *self;
			new_game.add_random_value(new_game.blanks());
			new_game.play_random_game();
			total_score += new_game.score();
			num_games += 1;
		}
		total_score /= num_games;
		return total_score;
	}


	// Iterations is number of iterations PER MOVE.
	fn make_best_move(&mut self, iterations: i32, next_states: Vec<State>) {
		let mut best_score = 0;

		for state in next_states {
			let mut total_score = 0;
			for _ in 0..iterations {
				let mut new_game = state;
				new_game.add_random_value(new_game.blanks());
				new_game.play_random_game();
				total_score += new_game.score();
			}
			if best_score < total_score {
				*self = state;
				best_score = total_score;
			}
		}
	}

	fn play_best_game(&mut self, iterations: i32) {
		let mut next_states = self.next_states();
		while(next_states.len() > 0) {
			self.make_best_move(iterations, next_states);
			self.add_random_value(self.blanks());
			//println!("{}", self);
			next_states = self.next_states();
		}
	}

	fn timed_make_best_move(&mut self, duration: i32, next_states: Vec<State>) {
		let mut best_score = 0;
		let num_states = next_states.len() as i32;
		let duration_per_move = (duration / num_states) as u128;
		for state in next_states {
			let total_score = state.average_score_time(duration_per_move);
			if best_score < total_score {
				*self = state;
				best_score = total_score;
			}
		}
	}

	fn parallel_timed_make_best_move(&mut self, duration: i32, next_states: Vec<State>, num_threads: usize) {
		let mut best_score = 0;
		let num_states = next_states.len() as i32;
		let duration_per_move = (duration / num_states) as u128;
		for state in next_states {
			let mut total_score = 0;
			let mut threads = Vec::with_capacity(num_threads);
			for _ in 0..num_threads {
				threads.push(thread::spawn(move || {state.average_score_time(duration_per_move) }));
			}
			for thread in threads {
				total_score += thread.join().ok().unwrap();
			}
			if best_score < total_score {
				*self = state;
				best_score = total_score;
			}
		}
	}


	fn timed_play_best_game(&mut self, duration: i32) {
		let mut next_states = self.next_states();
		while(next_states.len() > 0) {
			self.timed_make_best_move(duration, next_states);
			self.add_random_value(self.blanks());
			println!("{}", self);
			next_states = self.next_states();
		}
	}

	fn parallel_timed_play_best_game(&mut self, duration: i32, num_threads: usize) {
		let mut next_states = self.next_states();
		while(next_states.len() > 0) {
			self.parallel_timed_make_best_move(duration, next_states, num_threads);
			self.add_random_value(self.blanks());
			println!("{}", self);
			next_states = self.next_states();
		}
	}
}



impl fmt::Display for State {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for row in 0..4 {
			for col in 0..4 {
				let val = self.tiles[4 * row + col];
				let mut int_val : i32 = 0;
				if (val != 0) {
					int_val = 1 << val;
				} 
				write!(f, "{:>width$},", int_val, width=4)?;
			}
			write!(f, "\n")?;
		}
		return write!(f, "");
	}
}


fn main() {
	for _ in 0..20 {
	    let mut my_state = State {tiles: [0; 16]};
	    my_state.add_random_value(my_state.blanks());
	    my_state.timed_play_best_game(5000);
	    println!("{}", my_state);
	}
}
