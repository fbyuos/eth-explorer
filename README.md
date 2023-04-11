# Rust Ethereum Explorer Project

Eth-Explorer is a Rust-based project that allows users to explore Ethereum blockchain data by visualizing transaction information using a web interface. This project is my first attempt at using Rust after one week of learning, coming from a C/C++ embedded background. I've strived to make the code as idiomatic Rust as possible.

## Features

### Server Side (Rust)

- Retrieve Ethereum blockchain blocks using [ethers-rs](https://github.com/gakonst/ethers-rs)
- Store structured data in MongoDB
- Utilize Actix server for communication with the JavaScript client

### Client Side (HTML/JavaScript)

- Send requests to the server to retrieve transaction numbers and values
- Visualize the data using [Chart.js](https://www.chartjs.org/)
- Display the latest 10 blocks/transactions

## Getting Started

These instructions will help you set up and run the Eth-Explorer project on your local machine.

### Prerequisites

- Install [Rust](https://www.rust-lang.org/tools/install) and Cargo (included with Rust)
- Install [MongoDB](https://docs.mongodb.com/manual/installation/)
- Install [Docker](https://docs.docker.com/desktop/install/mac-install/)

### Installation

1. Clone the repository:

```
git clone https://github.com/fbyuos/eth-explorer.git
```

2. Change to the project directory:

```
cd eth-explorer
```

3. Use the Makefile to create and run a local MongoDB instance using Docker (mongodb://localhost:27017):

```
make mongostart
```

4. Run the server:

```
make dev
```

or

```
cargo run
```

5. A menu will appear; choose "Downloading history data." This process may take some time as it downloads 500 blocks from the blockchain:

Choose 7.
Press Enter

6. Choose 8 to run the webserver with Actix. Press Enter.

7. Open the `frontend/ethscan.html` file in your browser to view the client-side interface.

## Contributing

I welcome any contributions, suggestions, or feedback on this project. Please feel free to open an issue or submit a pull request.

## License

This project is licensed under the [MIT License](https://opensource.org/licenses/MIT).
