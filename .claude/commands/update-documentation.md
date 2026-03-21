Update project documentation after implementing a feature or change.

## Instructions

Read `design-source/project-definition.md` first. Determine what was recently implemented from context or ask the developer.

### Process

For each documentation file, determine if it needs updating and produce the updated content:

1. **README.md** — Does the change affect setup instructions, usage examples, API surface, or feature list?
2. **CHANGELOG.md** — Add an entry under the current version using Keep a Changelog format:
   ```
   ## [version] - YYYY-MM-DD
   ### Added / Changed / Fixed / Removed
   - Description of the change
   ```
3. **API Documentation** — If public interfaces changed, update relevant rustdoc
4. **ARCHITECTURE.md** — If project structure, module organization, or patterns changed
5. **Other** — Any other docs that should be updated

For each file, clearly indicate what was added or modified.
