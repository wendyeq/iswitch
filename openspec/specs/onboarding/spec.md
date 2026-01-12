# onboarding Specification

## Purpose
TBD - created by archiving change add-onboarding-tooltip. Update Purpose after archive.
## Requirements
### Requirement: First-Launch Proxy Enablement
The application MUST automatically enable the proxy mode for all available providers upon the user's very first launch.

#### Scenario: Fresh Installation
- **Given** the user has never opened the application before (no `localStorage` record)
- **When** the application loads the Main page
- **Then** the "Lightning" icon should be active (Blue/On state)
- **And** the backend proxy service should be enabled for 'claude' and 'codex'
- **And** a notification "Proxy Auto-Enabled" should appear briefly

### Requirement: Onboarding Tooltip
The application MUST display a visual guide pointing to the Proxy Toggle button on the first launch.

#### Scenario: Tooltip Display
- **Given** the application is in "First Launch" mode
- **When** the Capsule Navigation renders
- **Then** a "Ready" tooltip appears above the Lightning icon
- **And** the tooltip has a breathing animation to attract attention

#### Scenario: Tooltip Dismissal
- **Given** the tooltip is visible
- **When** the user clicks the Lightning icon OR clicks the tooltip itself
- **Then** the tooltip should disappear permanently
- **And** it should not reappear on subsequent app launches

### Requirement: Onboarding Persistence
The application MUST remember that the user has been onboarded.

#### Scenario: Second Launch
- **Given** the user has already seen the onboarding flow
- **When** the user reloads or restarts the application
- **Then** the proxy state respects the last saved state (or default behavior)
- **And** the onboarding tooltip DOES NOT appear

