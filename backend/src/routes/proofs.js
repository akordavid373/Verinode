const express = require('express');
const router = express.Router();
const { body, validationResult } = require('express-validator');
const StellarSDK = require('@stellar/stellar-sdk');

// Mock storage - replace with database
let proofs = [];
let proofIdCounter = 1;

// Issue a new proof
router.post('/issue', [
  body('eventData').notEmpty().withMessage('Event data is required'),
  body('hash').isLength({ min: 64 }).withMessage('Hash must be at least 64 characters')
], async (req, res) => {
  try {
    const errors = validationResult(req);
    if (!errors.isEmpty()) {
      return res.status(400).json({ errors: errors.array() });
    }

    const { eventData, hash, issuerAddress } = req.body;
    
    const proof = {
      id: proofIdCounter++,
      issuer: issuerAddress,
      eventData,
      hash,
      timestamp: new Date().toISOString(),
      verified: false,
      stellarTxId: null
    };

    proofs.push(proof);
    
    res.status(201).json({
      success: true,
      proof,
      message: 'Proof issued successfully'
    });
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

// Get proof by ID
router.get('/:id', (req, res) => {
  const proof = proofs.find(p => p.id === parseInt(req.params.id));
  if (!proof) {
    return res.status(404).json({ error: 'Proof not found' });
  }
  res.json({ proof });
});

// Get all proofs
router.get('/', (req, res) => {
  const { issuer, verified } = req.query;
  let filteredProofs = proofs;
  
  if (issuer) {
    filteredProofs = filteredProofs.filter(p => p.issuer === issuer);
  }
  
  if (verified !== undefined) {
    filteredProofs = filteredProofs.filter(p => p.verified === (verified === 'true'));
  }
  
  res.json({ proofs: filteredProofs });
});

// Verify a proof
router.post('/verify/:id', async (req, res) => {
  try {
    const proof = proofs.find(p => p.id === parseInt(req.params.id));
    if (!proof) {
      return res.status(404).json({ error: 'Proof not found' });
    }

    // Mock verification - implement actual Stellar transaction
    proof.verified = true;
    proof.verifiedAt = new Date().toISOString();
    
    res.json({
      success: true,
      proof,
      message: 'Proof verified successfully'
    });
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

module.exports = router;
