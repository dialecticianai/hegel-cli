# Customizing Workflows and Guides

Hegel supports user-defined workflows and guides that can extend or override the embedded defaults.

## Template Engines

Hegel supports two template engines for workflow prompts and guides:

**Markdown templates** (legacy, simple):
- Files: `*.md` in `guides/` and `guides/templates/`
- Syntax: `{{GUIDE_NAME}}`, `{{templates/name}}`, `{{variable}}`
- Use in workflows: `prompt:` field
- Best for: Simple guide inclusion and variable substitution

**Handlebars templates** (new, expressive):
- Files: `*.hbs` in `guides/partials/` and `guides/`
- Syntax: `{{> partial}}`, `{{context.variable}}`, `{{#if (eq ...)}}`, `{{#each}}`
- Use in workflows: `prompt_hbs:` field
- Best for: Conditional logic, loops, and dynamic content

**Load precedence:** `guides/partials/*.hbs` > `guides/*.hbs` > `guides/*.md` > `guides/templates/*.md`

This allows gradual migration—new templates can use Handlebars while existing Markdown templates continue working.

**Create custom workflows:**
```bash
# Create workflows directory
mkdir -p .hegel/workflows

# Markdown template example (simple)
cat > .hegel/workflows/my-workflow.yaml <<EOF
mode: discovery
start_node: start
nodes:
  start:
    prompt: "{{MY_GUIDE}}"
    transitions:
      - when: done
        to: done
  done:
    transitions: []
EOF

# Handlebars template example (with conditionals)
cat > .hegel/workflows/my-workflow.yaml <<EOF
mode: discovery
start_node: start
nodes:
  start:
    prompt_hbs: |
      {{> my_partial}}

      {{#if context.detailed_mode}}
      Additional detailed instructions here.
      {{/if}}
    transitions:
      - when: done
        to: done
  done:
    transitions: []
EOF

# Use your custom workflow
hegel start my-workflow
```

**Override embedded workflows:**
```bash
# Copy embedded workflow to customize it
# (filesystem versions take priority over embedded)
cp workflows/discovery.yaml .hegel/workflows/discovery.yaml

# Edit .hegel/workflows/discovery.yaml to customize
# Hegel will now use your customized version!
```

**Create custom guides:**
```bash
# Create guides directory
mkdir -p .hegel/guides

# Markdown guide (simple)
echo "# My Custom Spec Guide" > .hegel/guides/MY_GUIDE.md
# Reference with {{MY_GUIDE}} in prompt: fields

# Handlebars partial (with logic)
mkdir -p .hegel/guides/partials
cat > .hegel/guides/partials/my_partial.hbs <<EOF
{{#if (eq context.mode "detailed")}}
Detailed instructions go here.
{{else}}
Brief instructions go here.
{{/if}}
EOF
# Reference with {{> my_partial}} in prompt_hbs: fields
```

**Override embedded guides:**
```bash
# Override the default SPEC_WRITING guide (Markdown)
cp guides/SPEC_WRITING.md .hegel/guides/SPEC_WRITING.md
# Edit .hegel/guides/SPEC_WRITING.md to match your style

# Override with Handlebars partial (takes precedence)
cp guides/partials/code_map.hbs .hegel/guides/partials/code_map.hbs
# Edit .hegel/guides/partials/code_map.hbs for custom logic
```

**Priority:** Filesystem (`.hegel/`) takes precedence over embedded resources, and Handlebars partials take precedence over Markdown guides, allowing you to customize any aspect of Hegel's workflows.
