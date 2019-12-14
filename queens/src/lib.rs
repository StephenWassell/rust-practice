fn valid(part: &[u8]) -> bool {
	let mut sum = 0usize;
	let mut sum_left = 0usize;
	let mut sum_right = 0usize;

	for i in 0..part.len() {
		sum |= 1 << part[i];
		sum_left |= 1 << (part[i] as usize + i);
		sum_right |= 1 << (part[i] as usize + part.len() - i);
	}

	sum.count_ones() as usize == part.len() &&
		sum_left.count_ones() as usize == part.len() &&
		sum_right.count_ones() as usize == part.len()				
}

fn queens_rec(mut board: Vec<u8>, row: usize, quiet: bool) -> usize {
	if row == board.len() {
		if !quiet {
			println!("{:?}", board);
		}
		1
	} else {
		let mut count = 0;
		for i in 0..board.len() {
			board[row] = i as u8;
			if valid(&board[0..row + 1]) {
				count += queens_rec(board.clone(), row + 1, quiet);
			}
		}
		count
	}
}

pub fn queens(size: usize, quiet: bool) -> usize {
	let board = vec![0u8; size];
	queens_rec(board, 0, quiet)
}
