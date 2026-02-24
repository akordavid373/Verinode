#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

class SecurityAudit {
  constructor() {
    this.config = require('../config/securityConfig.js');
    this.issues = [];
    this.warnings = [];
    this.passed = [];
  }

  async runAudit() {
    console.log('🔍 Starting Security Audit...\n');

    // Check file permissions
    await this.checkFilePermissions();
    
    // Check environment variables
    await this.checkEnvironmentVariables();
    
    // Check dependencies
    await this.checkDependencies();
    
    // Check code for security issues
    await this.checkCodeSecurity();
    
    // Check configuration
    await this.checkConfiguration();
    
    // Generate report
    this.generateReport();
  }

  async checkFilePermissions() {
    console.log('📁 Checking file permissions...');
    
    const sensitiveFiles = [
      'config/securityConfig.js',
      '.env',
      'package.json',
      'src/middleware/auth.ts'
    ];

    for (const file of sensitiveFiles) {
      try {
        const filePath = path.join(__dirname, '..', file);
        if (fs.existsSync(filePath)) {
          const stats = fs.statSync(filePath);
          const mode = (stats.mode & parseInt('777', 8)).toString(8);
          
          if (mode !== '600' && mode !== '644') {
            this.issues.push({
              type: 'file_permission',
              file,
              message: `File has permissive permissions: ${mode}`,
              severity: 'high'
            });
          } else {
            this.passed.push(`File permissions OK: ${file}`);
          }
        }
      } catch (error) {
        this.warnings.push(`Could not check permissions for ${file}: ${error.message}`);
      }
    }
  }

  async checkEnvironmentVariables() {
    console.log('🔧 Checking environment variables...');
    
    const requiredEnvVars = [
      'NODE_ENV',
      'JWT_SECRET',
      'SESSION_SECRET'
    ];

    const recommendedEnvVars = [
      'ALLOWED_ORIGINS',
      'LOG_LEVEL',
      'LOG_TO_FILE'
    ];

    for (const envVar of requiredEnvVars) {
      if (!process.env[envVar]) {
        this.issues.push({
          type: 'environment',
          variable: envVar,
          message: `Required environment variable not set: ${envVar}`,
          severity: 'high'
        });
      } else {
        this.passed.push(`Environment variable set: ${envVar}`);
      }
    }

    for (const envVar of recommendedEnvVars) {
      if (!process.env[envVar]) {
        this.warnings.push(`Recommended environment variable not set: ${envVar}`);
      } else {
        this.passed.push(`Environment variable set: ${envVar}`);
      }
    }

    // Check for default secrets
    if (process.env.JWT_SECRET === 'your-jwt-secret') {
      this.issues.push({
        type: 'security',
        message: 'JWT_SECRET is using default value',
        severity: 'critical'
      });
    }

    if (process.env.SESSION_SECRET === 'your-secret-key') {
      this.issues.push({
        type: 'security',
        message: 'SESSION_SECRET is using default value',
        severity: 'critical'
      });
    }
  }

  async checkDependencies() {
    console.log('📦 Checking dependencies...');
    
    try {
      const packageJson = JSON.parse(fs.readFileSync(path.join(__dirname, '../package.json'), 'utf8'));
      const dependencies = { ...packageJson.dependencies, ...packageJson.devDependencies };
      
      // Check for known vulnerable packages (simplified check)
      const vulnerablePackages = [
        'lodash<4.17.21',
        'express<4.17.0',
        'helmet<4.0.0'
      ];

      for (const vulnPackage of vulnerablePackages) {
        const [name, version] = vulnPackage.split('<');
        if (dependencies[name]) {
          this.warnings.push(`Check if ${name} version is safe: ${dependencies[name]}`);
        }
      }

      // Check for security-related packages
      const securityPackages = ['helmet', 'cors', 'express-rate-limit'];
      for (const pkg of securityPackages) {
        if (dependencies[pkg]) {
          this.passed.push(`Security package found: ${pkg}`);
        } else {
          this.warnings.push(`Security package not found: ${pkg}`);
        }
      }

    } catch (error) {
      this.warnings.push(`Could not check dependencies: ${error.message}`);
    }
  }

  async checkCodeSecurity() {
    console.log('🔍 Checking code for security issues...');
    
    const securityPatterns = [
      {
        pattern: /eval\s*\(/,
        message: 'Use of eval() detected',
        severity: 'high'
      },
      {
        pattern: /Function\s*\(/,
        message: 'Use of Function() constructor detected',
        severity: 'high'
      },
      {
        pattern: /innerHTML\s*=/,
        message: 'Use of innerHTML detected (potential XSS)',
        severity: 'medium'
      },
      {
        pattern: /document\.write/,
        message: 'Use of document.write detected (potential XSS)',
        severity: 'medium'
      },
      {
        pattern: /console\.log/,
        message: 'Console.log statement detected (should be removed in production)',
        severity: 'low'
      }
    ];

    const sourceFiles = this.getSourceFiles();
    
    for (const file of sourceFiles) {
      try {
        const content = fs.readFileSync(file, 'utf8');
        
        for (const { pattern, message, severity } of securityPatterns) {
          if (pattern.test(content)) {
            this.issues.push({
              type: 'code',
              file: path.relative(process.cwd(), file),
              message,
              severity
            });
          }
        }
      } catch (error) {
        this.warnings.push(`Could not scan file ${file}: ${error.message}`);
      }
    }
  }

  async checkConfiguration() {
    console.log('⚙️ Checking security configuration...');
    
    // Check rate limiting configuration
    if (this.config.rateLimiting.api.max > 1000) {
      this.warnings.push('API rate limit is very high (>1000 requests)');
    }

    if (this.config.rateLimiting.auth.max > 20) {
      this.warnings.push('Auth rate limit is high (>20 attempts)');
    }

    // Check CORS configuration
    if (this.config.cors.production.origin === true) {
      this.issues.push({
        type: 'configuration',
        message: 'CORS allows all origins in production',
        severity: 'high'
      });
    }

    // Check session configuration
    if (!this.config.session.cookie.secure && this.config.environment.isProduction) {
      this.issues.push({
        type: 'configuration',
        message: 'Session cookies not secure in production',
        severity: 'high'
      });
    }

    // Check JWT configuration
    if (this.config.jwt.expiresIn > '24h') {
      this.warnings.push('JWT expiration time is longer than 24 hours');
    }

    this.passed.push('Security configuration checked');
  }

  getSourceFiles() {
    const srcDir = path.join(__dirname, '../src');
    const files = [];

    function scanDirectory(dir) {
      const items = fs.readdirSync(dir);
      
      for (const item of items) {
        const fullPath = path.join(dir, item);
        const stat = fs.statSync(fullPath);
        
        if (stat.isDirectory()) {
          scanDirectory(fullPath);
        } else if (item.endsWith('.ts') || item.endsWith('.js')) {
          files.push(fullPath);
        }
      }
    }

    if (fs.existsSync(srcDir)) {
      scanDirectory(srcDir);
    }

    return files;
  }

  generateReport() {
    console.log('\n📊 Security Audit Report\n');
    console.log('='.repeat(50));

    if (this.issues.length > 0) {
      console.log('\n🚨 Critical Issues Found:');
      this.issues
        .filter(issue => issue.severity === 'critical')
        .forEach(issue => {
          console.log(`  ❌ ${issue.message}`);
          if (issue.file) console.log(`     File: ${issue.file}`);
        });

      console.log('\n⚠️ High Severity Issues:');
      this.issues
        .filter(issue => issue.severity === 'high')
        .forEach(issue => {
          console.log(`  ⚠️ ${issue.message}`);
          if (issue.file) console.log(`     File: ${issue.file}`);
        });

      console.log('\n⚡ Medium Severity Issues:');
      this.issues
        .filter(issue => issue.severity === 'medium')
        .forEach(issue => {
          console.log(`  ⚡ ${issue.message}`);
          if (issue.file) console.log(`     File: ${issue.file}`);
        });
    }

    if (this.warnings.length > 0) {
      console.log('\n💡 Warnings:');
      this.warnings.forEach(warning => {
        console.log(`  💡 ${warning}`);
      });
    }

    if (this.passed.length > 0) {
      console.log('\n✅ Passed Checks:');
      this.passed.slice(0, 10).forEach(passed => {
        console.log(`  ✅ ${passed}`);
      });
      
      if (this.passed.length > 10) {
        console.log(`  ... and ${this.passed.length - 10} more`);
      }
    }

    console.log('\n' + '='.repeat(50));
    console.log(`📈 Summary:`);
    console.log(`  Critical Issues: ${this.issues.filter(i => i.severity === 'critical').length}`);
    console.log(`  High Issues: ${this.issues.filter(i => i.severity === 'high').length}`);
    console.log(`  Medium Issues: ${this.issues.filter(i => i.severity === 'medium').length}`);
    console.log(`  Low Issues: ${this.issues.filter(i => i.severity === 'low').length}`);
    console.log(`  Warnings: ${this.warnings.length}`);
    console.log(`  Passed: ${this.passed.length}`);

    const totalIssues = this.issues.length;
    if (totalIssues === 0) {
      console.log('\n🎉 No critical security issues found!');
    } else {
      console.log(`\n🔧 ${totalIssues} issue(s) need to be addressed.`);
    }

    // Exit with appropriate code
    process.exit(totalIssues > 0 ? 1 : 0);
  }
}

// Run the audit if called directly
if (require.main === module) {
  const audit = new SecurityAudit();
  audit.runAudit().catch(error => {
    console.error('Audit failed:', error);
    process.exit(1);
  });
}

module.exports = SecurityAudit;
