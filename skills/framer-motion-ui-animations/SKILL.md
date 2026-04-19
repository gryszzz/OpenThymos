---
name: framer-motion-ui-animations
description: Use when building or refining React/Next.js interfaces with Framer Motion or Motion for React, including page transitions, staggered reveals, shared layout animation, scroll-linked motion, hover/tap/drag interactions, and animation systems that need to feel premium without becoming janky.
---

# Framer Motion UI Animations

Use this skill when the user wants:

- Framer Motion or Motion for React
- animated React or Next.js UI
- premium page transitions
- staggered reveals or section choreography
- shared element or layout animation
- scroll-linked motion or parallax
- hover, tap, drag, and gesture-heavy interactions

For greenfield React work, prefer the current Motion for React package:

- install with `npm install motion`
- import from `"motion/react"`

This matches Motion's current React docs on `motion.dev`. If the repo already uses `framer-motion`, stay in the existing package unless the user explicitly asks to migrate.

Read [references/motion-patterns.md](references/motion-patterns.md) when implementing or reviewing motion.

## Core Goal

Make the interface feel expensive, responsive, and intentional.

The motion should:

- reveal hierarchy
- reinforce depth
- preserve orientation between states
- stay performant on real hardware
- remain understandable in code

## Workflow

### 1. Choose the Motion Job

Before writing animation code, classify the request:

- **entry/reveal**: section intros, staggered cards, hero buildup
- **state transition**: opening panels, tabs, drawers, modals
- **layout transition**: cards expanding, reordering, shared element motion
- **scroll motion**: progress bars, parallax, section-driven transforms
- **gesture interaction**: hover lift, tap compression, drag surfaces

Use the simplest mechanism that matches the job.

### 2. Pick the Right Motion Primitive

- use `variants` for coordinated enter/reveal systems
- use `AnimatePresence` for enter/exit transitions
- use `layout`, `layoutId`, and `LayoutGroup` for layout continuity
- use `whileHover`, `whileTap`, and `drag` for interaction states
- use `whileInView` for scroll-triggered reveals
- use `useScroll`, `useTransform`, and `useSpring` for scroll-linked motion
- use `useReducedMotion` whenever motion intensity needs an accessible fallback

### 3. Choreograph Like a System

Prefer:

- one strong hero motion idea
- one repeatable stagger pattern for supporting content
- one shared transition language for cards and overlays

Avoid:

- every element animating independently
- mixing too many animation idioms in one surface
- long cinematic delays that block the interface

### 4. Keep It Premium, Not Sloppy

- default to transform and opacity first
- use springs for physical surfaces and tweens for simpler fades
- keep ambient loops subtle and low-frequency
- treat blur, clip-path, and filter animation as luxury accents, not defaults
- define reduced-motion behavior instead of simply deleting the component

### 5. Output Contract

When using this skill, produce:

1. the motion roles in the UI
2. the primitives to use (`variants`, `AnimatePresence`, `layout`, etc.)
3. the implementation pattern
4. the reduced-motion fallback
5. the performance notes if the surface is visually ambitious

## Preferred Patterns

### Staggered Reveals

- parent variants own timing
- children inherit via `staggerChildren` and `delayChildren`
- combine slight `y`/`scale` change with opacity rather than large travel distances

### Shared Layout

- use `layout` for local size/position continuity
- use `layoutId` when an element visually transforms into another element
- preserve border radius and surface identity during the transition

### Scroll Motion

- use `whileInView` for section reveals
- use `useScroll` + `useTransform` for linked values
- smooth reactive values with `useSpring` when direct mapping feels brittle

### Interaction Motion

- hover should feel like lift, sheen, tilt, or soft expansion
- tap should compress slightly and release quickly
- drag should feel weighted, not elastic by default

## Pairing Guidance

Pair this skill with:

- `$ui-immersive-3d-dribbble` when the user wants premium, showcase-level visuals
- `fixing-motion-performance` when the repo already has janky or expensive animation behavior

The Dribbble skill should decide the visual thesis.
This skill should decide how that thesis moves in React.
