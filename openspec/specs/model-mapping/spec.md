# model-mapping Specification

## Purpose
TBD - created by archiving change enable-model-mapping. Update Purpose after archive.
## Requirements
### Requirement: Support Wildcard and Exact Model Mapping

The system MUST allow users to map input model names to different output model names using both exact matches and wildcard patterns to support custom providers that require specific model names.

#### Scenario: Exact Mapping
Given a provider is configured with mapping `{"claude-3-opus": "anthropic/claude-3-opus"}`
When the client sends a message request with `model: "claude-3-opus"`
Then the proxy MUST forward the request with `model: "anthropic/claude-3-opus"` in the JSON body.

#### Scenario: Wildcard Mapping
Given a provider is configured with mapping `{"claude-*": "my-claude-*"}`
When the client sends a message request with `model: "claude-3-sonnet"`
Then the proxy MUST forward the request with `model: "my-claude-3-sonnet"` in the JSON body.

