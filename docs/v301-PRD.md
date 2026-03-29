# PRD

## User Journeys

### User Journey 01 : Tauri App for indexing a folder and providing a HTTP URL to the LLM

``` text

Step 01: User downloads the Tauri App from github releases of the github repository - since we will limit it to macOS only for now, it will be a .dmg file

Step 02: User opens the Tauri App and is presented with a welcome screen, which tells the user that this app is privacy-first and does not send any data to any server, everything happens locally on the user's machine

Step 03: User sees 
    - Assuming workspaces are indexed, the user is presented with a table view of workspaces
        - top most row is Add new workspace button (click to add a new workspace)
        - each workspace row has the following information:
            - workspace name
            - workspace status
            - workspace last indexed
            - reindex button
            - start HTTP server button
            - stop HTTP server button
            - delete workspace button
            - copy HTTP URL button (click to copy the HTTP URL to the clipboard)


```


# Raw data for pointers


``` text

  │     │ Tauri Mac app with workspace management             │     │      │ One-click from "I have code" to "my LLM understands it." No CLI. No config       │                                              │
  │ 13  │ (drag-and-drop folder, progress bar, FRESH/STALE    │ 91  │ 70   │ files. No Docker. Download .dmg, open, drag folder. The HTTP URL is the only     │ PRD-v300 (Screens 1-7), FUJ (Phase 1)        │
  │     │ badge, HTTP URL copy)                               │     │      │ thing the LLM needs.                                                             │                                              │
  ├─────┼─────────────────────────────────────────────────────┼─────┼──────┼──────────────────────────────────────────────────────────────────────────────────┼───────────────────────────────────────



```



