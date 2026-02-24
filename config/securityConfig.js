module.exports = {
  // Rate limiting configuration
  rateLimiting: {
    // General API rate limiting
    api: {
      windowMs: 15 * 60 * 1000, // 15 minutes
      max: 100, // 100 requests per window
      message: 'Rate limit exceeded. Please try again later.',
      standardHeaders: true,
      legacyHeaders: false
    },
    
    // Authentication endpoints
    auth: {
      windowMs: 15 * 60 * 1000, // 15 minutes
      max: 5, // 5 attempts per window
      message: 'Too many authentication attempts. Please try again later.'
    },
    
    // GraphQL endpoints
    graphql: {
      query: {
        windowMs: 1 * 60 * 1000, // 1 minute
        max: 60 // 60 queries per minute
      },
      mutation: {
        windowMs: 1 * 60 * 1000, // 1 minute
        max: 30 // 30 mutations per minute
      }
    },
    
    // File upload endpoints
    upload: {
      windowMs: 60 * 60 * 1000, // 1 hour
      max: 10 // 10 uploads per hour
    }
  },

  // CORS configuration
  cors: {
    development: {
      origin: true,
      methods: ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS'],
      allowedHeaders: '*',
      credentials: true
    },
    
    production: {
      origin: process.env.ALLOWED_ORIGINS?.split(',') || ['https://yourdomain.com'],
      methods: ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS'],
      allowedHeaders: [
        'Origin',
        'X-Requested-With',
        'Content-Type',
        'Accept',
        'Authorization',
        'Cache-Control',
        'X-API-Key'
      ],
      credentials: true,
      maxAge: 86400 // 24 hours
    },
    
    strict: {
      origin: process.env.ALLOWED_ORIGINS?.split(',') || ['https://yourdomain.com'],
      methods: ['GET', 'POST'],
      allowedHeaders: [
        'Origin',
        'Content-Type',
        'Accept',
        'Authorization'
      ],
      credentials: true
    }
  },

  // Security headers configuration
  securityHeaders: {
    contentSecurityPolicy: {
      directives: {
        defaultSrc: ["'self'"],
        styleSrc: [
          "'self'",
          "'unsafe-inline'",
          "https://fonts.googleapis.com"
        ],
        fontSrc: [
          "'self'",
          "https://fonts.gstatic.com"
        ],
        imgSrc: [
          "'self'",
          "data:",
          "https:"
        ],
        scriptSrc: [
          "'self'"
        ],
        connectSrc: [
          "'self'",
          "https://api.yourdomain.com"
        ],
        frameSrc: ["'none'"],
        objectSrc: ["'none'"],
        mediaSrc: ["'self'"],
        manifestSrc: ["'self'"],
        workerSrc: ["'self'"]
      }
    },
    
    customHeaders: {
      'X-Content-Type-Options': 'nosniff',
      'X-Frame-Options': 'DENY',
      'X-XSS-Protection': '1; mode=block',
      'Referrer-Policy': 'strict-origin-when-cross-origin',
      'Permissions-Policy': 'geolocation=(), microphone=(), camera=(), payment=(), usb=(), magnetometer=(), gyroscope=(), accelerometer=()'
    }
  },

  // Input validation configuration
  validation: {
    maxPayloadSize: 10 * 1024 * 1024, // 10MB
    allowedContentTypes: [
      'application/json',
      'application/x-www-form-urlencoded',
      'multipart/form-data',
      'text/plain'
    ],
    
    // Field-specific validation rules
    fields: {
      email: {
        maxLength: 255,
        pattern: /^[^\s@]+@[^\s@]+\.[^\s@]+$/
      },
      username: {
        minLength: 3,
        maxLength: 50,
        pattern: /^[a-zA-Z0-9_-]+$/
      },
      password: {
        minLength: 8,
        maxLength: 128,
        pattern: /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]/
      },
      id: {
        pattern: /^[a-zA-Z0-9_-]+$/
      }
    }
  },

  // XSS protection configuration
  xssProtection: {
    allowedTags: ['b', 'i', 'em', 'strong', 'p', 'br', 'span'],
    allowedAttributes: ['class'],
    allowScriptTags: false,
    allowStyleTags: false
  },

  // Request logging configuration
  logging: {
    maxLogs: 10000,
    logLevel: process.env.LOG_LEVEL || 'info',
    logToFile: process.env.LOG_TO_FILE === 'true',
    logFilePath: process.env.LOG_FILE_PATH || './logs/security.log',
    
    // Security monitoring
    securityMonitoring: {
      enableAlerts: true,
      alertThreshold: {
        suspiciousIP: 10, // Alert after 10 suspicious requests from same IP
        failedAuth: 5, // Alert after 5 failed auth attempts
        xssAttempts: 3, // Alert after 3 XSS attempts
        sqlInjectionAttempts: 1 // Immediate alert for SQL injection attempts
      }
    }
  },

  // Database security configuration
  database: {
    connectionTimeout: 30000, // 30 seconds
    queryTimeout: 10000, // 10 seconds
    maxConnections: 100,
    
    // SQL injection prevention
    sqlInjection: {
      enableParameterizedQueries: true,
      validateQueries: true,
      logSuspiciousQueries: true
    }
  },

  // Session security configuration
  session: {
    secret: process.env.SESSION_SECRET || 'your-secret-key',
    resave: false,
    saveUninitialized: false,
    rolling: true,
    cookie: {
      secure: process.env.NODE_ENV === 'production',
      httpOnly: true,
      maxAge: 24 * 60 * 60 * 1000, // 24 hours
      sameSite: 'strict'
    }
  },

  // JWT configuration
  jwt: {
    secret: process.env.JWT_SECRET || 'your-jwt-secret',
    expiresIn: '24h',
    issuer: 'verinode',
    audience: 'verinode-users'
  },

  // Environment-specific settings
  environment: {
    isProduction: process.env.NODE_ENV === 'production',
    isDevelopment: process.env.NODE_ENV === 'development',
    isTest: process.env.NODE_ENV === 'test'
  }
};
