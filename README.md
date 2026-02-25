# Verinode

Web3 infrastructure for issuing and verifying cryptographic proofs on Stellar.

## Prerequisites

- **Node.js** >= 16.0.0 (LTS version recommended)
- **npm** >= 8.0.0
- **Rust** and **Cargo** (for smart contracts)
- **Docker** and **Docker Compose** (optional, for containerized deployment)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/Great-2025/Verinode.git
cd Verinode
```

2. Install all dependencies:
```bash
npm run install:all
```

This will install dependencies for:
- Root project
- Backend API
- Frontend React application
- Smart contracts (via Cargo)

## Development

Start the development environment:
```bash
npm run dev
```

This will concurrently start:
- Backend development server (http://localhost:3001)
- Frontend development server (http://localhost:3000)

## Building

Build all components:
```bash
npm run build
```

Or build individual components:
```bash
npm run build:contracts  # Build smart contracts
npm run build:backend    # Build backend API
npm run build:frontend   # Build frontend app
```

## Testing

Run all tests:
```bash
npm test
```

Or test individual components:
```bash
npm run test:contracts  # Test smart contracts
npm run test:backend    # Test backend API
npm run test:frontend   # Test frontend app
```

## Deployment

### Local Deployment
```bash
npm run deploy:local
```

### Testnet Deployment
```bash
npm run deploy:testnet
```

### Docker Deployment
```bash
docker-compose up -d
```

## Security

This project uses modern, secure dependencies:
- No deprecated packages (e.g., `request` package has been replaced with `axios`)
- All dependencies are regularly updated for security patches
- Node.js 16+ LTS required for latest security features
- Security headers implemented via Helmet middleware
- Rate limiting and input validation

## Architecture

- **Backend**: Node.js/Express API with Stellar SDK integration
- **Frontend**: React with TypeScript and Tailwind CSS
- **Smart Contracts**: Rust-based Soroban contracts on Stellar
- **Storage**: IPFS for decentralized file storage
- **Database**: MongoDB for metadata and audit trails

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- 📖 [Documentation](docs/)
- 🐛 [Issue Tracker](https://github.com/Great-2025/Verinode/issues)
- 💬 [Discussions](https://github.com/Great-2025/Verinode/discussions)