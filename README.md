# sudoku-rs
A Sudoku solver written in Rust

## build
```
rustfmt sudoku.rs && rustc sudoku.rs
```

## tests
```
easy:
echo "3..9..7.11....45.9984........9.268..4...9...5..241.6........4122.38....76.1..9..8" | ./sudoku

evil:
echo "...6....17...945..4....2....5..1.7.2.2.....6.3.6.8..9....8....7..376...89....3..." | ./sudoku
```