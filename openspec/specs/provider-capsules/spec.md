# provider-capsules Specification

## Purpose
TBD - created by archiving change refactor-provider-capsules. Update Purpose after archive.
## Requirements
### Requirement: Visual Theme - Ocean Depth (Dark Mode)
The provider list MUST use the "Ocean Depth" theme in Dark Mode.
- **Background**: Deep Emerald Teal gradient.
- **Capsule**: Dark frosted glass, Ice Blue glowing borders for Active state.
- **Flow Line**: Bioluminescent Cyan.

#### Scenario: Dark Mode Visualization
- **Given** the app is in Dark Mode
- **When** the user views the provider list
- **Then** the background should be dark emerald/teal
- **And** the capsules should look like frosted glass with ice blue accents.

### Requirement: Visual Theme - Milky Glass (Light Mode)
The provider list MUST use the "Milky Glass" theme in Light Mode.
- **Background**: Azure -> Teal light gradient.
- **Capsule**: Milky white frosted glass, Sunny Yellow accents for cost/active toggle.
- **Text**: High contrast Slate/Teal.

#### Scenario: Light Mode Visualization
- **Given** the app is in Light Mode
- **When** the user views the provider list
- **Then** the background should be a light azure/teal gradient
- **And** the capsules should look like milky white ceramic/glass.

### Requirement: Levitating Sorting
The provider list MUST support vertical drag-and-drop sorting to adjust priority.
- **Interaction**: Drag handle on the right or long-press capsule.
- **Visual**: Dragged item "lifts" and casts a larger shadow.
- **Persistence**: New order is saved to backend immediately or on drop.

#### Scenario: Reorder Priority
- **Given** a list of 5 providers
- **When** the user drags the 3rd provider to the 1st position
- **Then** the provider should stay at the 1st position
- **And** the priority levels should efficiently update in the backend.

### Requirement: Expandable Details
Each capsule MUST be expandable to show detailed metrics in a "Dashboard" layout.
- **Collapsed**: Icon, Name, Mini "Success %" Badge (Green Pill), Toggle, Drag Handle.
- **Expanded**: Header remains + Dashboard Body appears below.
- **Dashboard**: A distinct inner panel containing 4 data blocks.
- **Action**: Clicking the capsule body toggles expansion.

#### Scenario: Expand Capsule
- **Given** a collapsed provider capsule
- **When** the user clicks on the capsule body
- **Then** the capsule should expand to reveal the Dashboard
- **And** the 4-card metric grid should be visible.

### Requirement: Data Metrics Display
The expanded Dashboard view MUST display 4 distinct blocks:
1. **Success Rate**: Large Ring Gauge (Green/Yellow/Red) + Percentage.
2. **Requests**: Total Count + Tiny "Sparkline" trend graph representing **Hourly Request Volume (Last 24h)** using REAL time-series data.
3. **Tokens**: Total Count + Visual Wave/Particle effect.
4. **Cost**: Total Cost highlighted in Gold/Yellow color.

#### Scenario: Verify Metrics Data
- **Given** a provider has usage data
- **When** the capsule is expanded
- **Then** the dashboard should appear with the 4 blocks described above.
- **And** the Cost should be highlighted in Gold/Yellow style.

#### Scenario: Sparkline Real Data
- **Given** the provider has request history in the last 24 hours
- **When** the capsule is expanded
- **Then** the Sparkline should reflect the actual distribution of requests over time
- **And** SHOULD NOT be random visual decoration.

### Requirement: Smart Active Indicator (Auto-Highlight)
The provider list MUST visually indicate which provider is currently "Active" (handling traffic) via a distinct Halo effect.
- **Visual**: A glowing Blue Halo (aura) around the active provider capsule.
- **Logic Priority**:
    1.  **Session Dominance**: The provider with the highest number of **successful requests (200 OK)** within the **current session's last 20 logs**.
    2.  **Visual Fallback**: If no session logs exist (e.g., fresh start), default to the **First Enabled Provider**.
- **Session Reset**: On App Restart or Page Reload, the state MUST reset to the First Provider (ignoring historical logs from DB).
- **Failover**: If the active provider fails (no recent successes) and another provider succeeds, the Halo MUST automatically jump to the new successful provider.

#### Scenario: Failover Visualization
- **Given** Provider A is active (Halo)
- **When** Provider A fails and Provider B successfully handles the next request
- **Then** the Blue Halo should move from Provider A to Provider B.

#### Scenario: Session Reset
- **Given** Provider B was active in the previous session
- **When** the application is restarted
- **Then** the Blue Halo should reset to Provider A (First) by default.

### Requirement: Component Replacement
The existing `automation-list` grid/list in `Index.vue` MUST be replaced entirely by the new `LevitatingProviderList` component.

#### Scenario: Verify Replacement
- **Given** the user is on the Homepage
- **When** the page loads
- **Then** the old card list should not be visible
- **And** the new Levitating Capsule list should be displayed.

### Requirement: Smart Official Site Link
The capsule actions MUST include a link to the provider's official dashboard or website.
- **Icon**: Compass / Globe icon.
- **Priority Logic**:
    1. **Explicit**: Use `officialSite` field if manually configured.
    2. **Type Default**: Use preset dashboards for known types (e.g., Zhipu -> bigmodel.cn, OpenAI -> platform.openai.com).
    3. **Auto-Detect**: Attempt to resolve dashboard from `apiUrl` (e.g., `api.deepseek.com` -> `platform.deepseek.com`).
    4. **Origin Fallback**: Use the `apiUrl` origin as a last resort.
- **Behavior**: Opens within the default system browser.

### Requirement: Auto-Switch Transparency
The system MUST be fully transparent about which provider is handling requests and whether it was selected manually or automatically.

#### 1. Status Badges ("Auto" vs "Manual")
Each provider capsule MUST display a status badge in the header when it is the **Active** provider:
- **Auto (Orange)**: Indicates the provider was automatically selected by the system (e.g., due to failover or session dominance).
    - **Visual**: Orange/Amber text with a small spinning sync icon.
    - **Tooltip**: "Auto Switch" or equivalent localized text.
- **Manual (Green)**: Indicates the provider was explicitly selected by the user (or is the default preference).
    - **Visual**: Emerald/Green text with a checkmark icon.
    - **Tooltip**: "Manual Select" or "Primary Choice".

#### 2. Inline Notification
When an automatic switch occurs (Failover or Session Dominance adjustment), the system MUST display a **temporary inline notification** to inform the user.
- **Location**: Top-center of the screen, floating above content.
- **Content**: "{Old Provider} unresponsive, switched to {New Provider}".
- **Duration**: Disappears automatically after 3 seconds.
- **Style**: Glassmorphism aesthetic, dark background, consistent with the app's premium feel.

#### Scenario: Visualizing Auto-Switch
- **Given** the system detects "Claude" is failing
- **When** it successfully switches traffic to "DeepSeek"
- **Then** the "DeepSeek" capsule should show an **Orange "Auto" Badge**
- **And** an inline notification should appear at the top saying "Claude unresponsive, switched to DeepSeek".
