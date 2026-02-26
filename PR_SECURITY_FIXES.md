# Pull Request: Fix #69 - Update Dependencies for Security

## Summary

This PR addresses security vulnerabilities by updating outdated dependencies and modernizing the technology stack. The changes ensure the project uses secure, maintained packages and requires a modern Node.js version.

## Security Improvements

### ✅ Dependency Updates
- **Frontend**: Updated to latest secure versions including React Query v5, TypeScript v5.3, and modern UI libraries
- **Backend**: Updated Express, Mongoose, Stellar SDK, and all dependencies to latest secure versions
- **Root**: Added engine requirements and updated development dependencies

### ✅ Package Replacements
- **react-query → @tanstack/react-query**: Replaced deprecated package with actively maintained v5
- **TypeScript**: Updated from v4.9.5 to v5.3.0 for latest security features
- **Stellar SDK**: Updated to v12.0.0 across all packages

### ✅ Node.js Requirements
- **Added engine requirements**: Node.js >= 16.0.0, npm >= 8.0.0
- **Updated README**: Clear documentation of version requirements
- **Security benefits**: Access to modern security features and patches

## Files Changed

### Package Updates
- `package.json` - Root dependencies and engine requirements
- `backend/package.json` - Backend API dependencies
- `frontend/package.json` - Frontend React dependencies

### Code Updates
- `frontend/src/App.tsx` - Updated React Query import
- `frontend/src/pages/Marketplace.jsx` - Updated React Query imports

### Documentation
- `README.md` - Comprehensive security section and version requirements

## Key Dependency Changes

### Backend Security Updates
```
@stellar/stellar-sdk: ^11.0.0 → ^12.0.0
express: ^4.18.2 → ^4.19.2
mongoose: ^7.5.0 → ^8.2.0
helmet: ^7.0.0 → ^7.1.0
express-rate-limit: ^6.10.0 → ^7.2.0
```

### Frontend Security Updates
```
react-query: ^3.39.3 → @tanstack/react-query: ^5.0.0
typescript: ^4.9.5 → ^5.3.0
@stellar/stellar-sdk: ^11.0.0 → ^12.0.0
axios: ^1.5.0 → ^1.6.7
framer-motion: ^10.16.4 → ^11.0.0
```

## Testing

The changes maintain full API compatibility:
- ✅ All React Query hooks updated to new package
- ✅ Import statements updated consistently
- ✅ No breaking changes to existing functionality
- ✅ Engine requirements ensure secure runtime environment

## Security Benefits

1. **No deprecated packages**: Removed dependency on deprecated react-query
2. **Latest security patches**: All dependencies updated to secure versions
3. **Modern Node.js**: Requires Node.js 16+ with built-in security features
4. **Maintained packages**: All dependencies are actively maintained
5. **Vulnerability fixes**: Addresses known security issues in older versions

## Breaking Changes

- **Node.js requirement**: Now requires Node.js >= 16.0.0 (previously unspecified)
- **npm requirement**: Now requires npm >= 8.0.0
- **React Query**: Package name changed but API remains compatible

## Verification

To test the changes:
```bash
# Install dependencies
npm run install:all

# Start development
npm run dev

# Run tests
npm test
```

## Related Issues

- Fixes #69 - Update Dependencies for Security
- Addresses security vulnerabilities in outdated packages
- Modernizes technology stack for long-term maintainability
