# Pull Request

## Description
<!-- Provide a clear description of what this PR does -->

## Type of Change
- [ ] Bug fix (non-breaking change fixing an issue)
- [ ] New feature (non-breaking change adding functionality)
- [ ] Breaking change (fix or feature causing existing functionality to change)
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Code refactoring
- [ ] CI/CD improvement
- [ ] Dependency update

## Related Issues
<!-- Link to related issues -->
Fixes #(issue)
Relates to #(issue)

## Changes Made
<!-- List specific changes -->
-
-
-

## Screenshots/Videos
<!-- If applicable, add screenshots or videos -->

## Testing Performed
<!-- Describe testing done -->

### Unit Tests
- [ ] Added unit tests for new code
- [ ] All unit tests pass
- [ ] Code coverage maintained/improved

### Integration Tests
- [ ] Added integration tests
- [ ] All integration tests pass

### Manual Testing
- [ ] Tested locally
- [ ] Tested in Docker
- [ ] Tested in production-like environment

### Test Commands Run
```bash
make test
make lint
make audit
# Add other commands you ran
```

## Performance Impact
<!-- Any performance implications? -->
- [ ] No performance impact
- [ ] Performance improved
- [ ] Performance decreased (explain why acceptable)

### Benchmarks
<!-- If applicable, include before/after benchmarks -->
```
Before:
After:
```

## Security Considerations
- [ ] No security impact
- [ ] Security improved
- [ ] Requires security review

### Security Checklist
- [ ] No hardcoded secrets
- [ ] Input validation added where needed
- [ ] Authentication/authorization checked
- [ ] SQL injection prevented
- [ ] XSS prevented
- [ ] CSRF protection maintained

## Breaking Changes
<!-- List any breaking changes and migration path -->
- [ ] No breaking changes
- [ ] Breaking changes (describe below)

**Migration Guide:**
```
<!-- If breaking changes, provide migration steps -->
```

## Documentation
- [ ] Code is self-documenting
- [ ] Inline documentation added/updated
- [ ] README.md updated
- [ ] ARCHITECTURE.md updated (if applicable)
- [ ] API documentation updated (if applicable)
- [ ] CHANGELOG.md updated

## Dependencies
<!-- Any new dependencies added? -->
- [ ] No new dependencies
- [ ] Dependencies added (list below)

**New Dependencies:**
- Dependency: version - reason

## Deployment Notes
<!-- Special deployment considerations? -->
- [ ] No special deployment steps
- [ ] Requires migration (describe below)
- [ ] Requires configuration changes (describe below)
- [ ] Requires database changes (describe below)

## Checklist
- [ ] My code follows the project's code style
- [ ] I have performed a self-review
- [ ] I have commented complex/hard-to-understand code
- [ ] I have made corresponding documentation changes
- [ ] My changes generate no new warnings
- [ ] I have added tests proving my fix/feature works
- [ ] New and existing tests pass locally
- [ ] Any dependent changes have been merged

### Code Quality
- [ ] Ran `make format` and committed formatted code
- [ ] Ran `make lint` and fixed all issues
- [ ] Ran `make audit` and addressed security concerns
- [ ] Ran `make test` and all tests pass

## Additional Notes
<!-- Any other information reviewers should know -->

---

**For Reviewers:**
- Please test this PR locally before approving
- Check for potential security issues
- Verify documentation is complete and accurate
- Ensure tests are comprehensive
