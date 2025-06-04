# Independent Time System

This project includes a complete independent time system that allows you to control the simulation speed independently from real time. The implementation provides precise control over virtual time flow with pause/resume capabilities and various timer utilities.

## Features

- **Speed Control**: Speed up or slow down the simulation (0.1x to 10x speed)
- **Pause/Resume**: Pause the simulation completely without losing time tracking
- **Virtual Time Tracking**: Keep track of simulation time independently from wall clock time
- **Multiple Timer Types**: One-shot timers, repeating timers, and utility functions
- **Fine-grained Control**: Precise control over time flow with smooth transitions
- **Time Formatting**: Human-readable time display (MM:SS.mmm format)

## Controls

### Keyboard Controls

- **1-6**: Set predefined speeds
  - `1`: 0.25x speed (quarter speed)
  - `2`: 0.5x speed (half speed)
  - `3`: 1.0x speed (normal speed)
  - `4`: 2.0x speed (double speed)
  - `5`: 4.0x speed (4x speed)
  - `6`: 8.0x speed (8x speed)

- **Arrow Keys**:
  - `↑`: Increase speed by 25% (max 10x)
  - `↓`: Decrease speed by 20% (min 0.1x)

- **R**: Reset time to zero
