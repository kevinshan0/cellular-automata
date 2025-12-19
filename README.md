# Cellular Automata

A high-performance, customizable cellular automata simulation built in Rust using the Nannou creative coding framework.

This engine separates simulation logic from rendering, allowing for complex, multi-state rulesets (beyond simple binary Game of Life) while maintaining high performance through optimized rendering techniques.

## Getting Started

### Prerequisites
You need rust and cargo installed on your machine

    curl --proto '=https' --tlsv1.2 -sSf [https://sh.rustup.rs](https://sh.rustup.rs) | sh

### Running the simulation

Clone this repository and run the project using Cargo.

Important: Always run with the --release flag. Cellular automata simulations involve millions of array checks per second, and Rust's debug mode is too slow for this.

    cargo run --release

## Configuration

### Changing colors
Open src/main.rs and look for the PALETTE constant at the top.
Index 0 is always the background (Dead/Empty state).


    const PALETTE: &[[u8; 4]] = &[
        [0, 0, 0, 255],       // State 0: Black
        [255, 65, 54, 255],   // State 1: Red
        [46, 204, 64, 255],   // State 2: Green
        // Add more colors here...
    ];

### Changing Ruleset
The simulation logic lives entirely in the solve_cell function in src/main.rs.
This function receives the current state of a cell and its 8 neighbors. It must return the state for the next frame.