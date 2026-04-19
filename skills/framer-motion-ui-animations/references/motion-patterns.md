# Motion Patterns

These notes support `$framer-motion-ui-animations`.

Official references used:

- Motion for React docs: https://motion.dev/docs/react
- Installation guide: https://motion.dev/docs/react-installation
- Upgrade guide: https://motion.dev/docs/react-upgrade-guide
- React animation overview: https://motion.dev/docs/react-animation
- Gesture docs: https://motion.dev/docs/react-gestures
- `useSpring`: https://motion.dev/docs/react-use-spring
- `useReducedMotion`: https://motion.dev/docs/react-use-reduced-motion

## Current Package Baseline

Motion's current React docs say:

- install with `npm install motion`
- import from `"motion/react"`
- React `18.2+` is required

The upgrade guide also documents moving from `framer-motion` to `motion`.

Practical rule for this skill:

- for greenfield work, prefer `motion`
- for existing repos already on `framer-motion`, do not migrate unless asked

## Pattern 1: Section Reveal System

Use this when the interface needs a crisp, premium entry pattern.

```tsx
import { motion } from "motion/react";

const section = {
  hidden: {},
  show: {
    transition: {
      staggerChildren: 0.08,
      delayChildren: 0.06,
    },
  },
};

const item = {
  hidden: { opacity: 0, y: 20, scale: 0.98 },
  show: {
    opacity: 1,
    y: 0,
    scale: 1,
    transition: { duration: 0.55, ease: [0.22, 1, 0.36, 1] },
  },
};

export function RevealGrid() {
  return (
    <motion.section variants={section} initial="hidden" animate="show">
      <motion.div variants={item} />
      <motion.div variants={item} />
      <motion.div variants={item} />
    </motion.section>
  );
}
```

## Pattern 2: Shared Layout Continuity

Use this for cards that expand into detail views, selected states, or morphing surfaces.

```tsx
import { LayoutGroup, motion } from "motion/react";

export function SharedCard({ active }: { active: boolean }) {
  return (
    <LayoutGroup>
      {active ? (
        <motion.div layoutId="feature-card" className="expanded" />
      ) : (
        <motion.button layoutId="feature-card" className="collapsed" />
      )}
    </LayoutGroup>
  );
}
```

Guidance:

- preserve corner radius and surface feel across both states
- avoid changing visual identity too aggressively mid-transition

## Pattern 3: Presence-Based UI

Use `AnimatePresence` for overlays, tooltips, drawers, and stateful screen transitions.

```tsx
import { AnimatePresence, motion } from "motion/react";

export function PresencePanel({ open }: { open: boolean }) {
  return (
    <AnimatePresence initial={false} mode="wait">
      {open ? (
        <motion.aside
          key="panel"
          initial={{ opacity: 0, y: 16 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 10 }}
          transition={{ duration: 0.32, ease: [0.22, 1, 0.36, 1] }}
        />
      ) : null}
    </AnimatePresence>
  );
}
```

## Pattern 4: Scroll-Linked Motion

Use this for premium hero depth, progress rails, and image parallax.

```tsx
import { motion, useScroll, useSpring, useTransform } from "motion/react";

export function ScrollRail() {
  const { scrollYProgress } = useScroll();
  const scaleX = useSpring(scrollYProgress, {
    stiffness: 180,
    damping: 28,
    mass: 0.25,
  });

  const y = useTransform(scrollYProgress, [0, 1], [0, -80]);

  return (
    <>
      <motion.div style={{ scaleX, transformOrigin: "0% 50%" }} />
      <motion.div style={{ y }} />
    </>
  );
}
```

Guidance:

- smooth raw scroll values with springs when needed
- keep the transform range restrained
- avoid turning every section into parallax

## Pattern 5: Gesture Polish

Use gestures for tactile cards, buttons, and control surfaces.

```tsx
import { motion } from "motion/react";

export function GestureCard() {
  return (
    <motion.button
      whileHover={{ y: -6, scale: 1.01 }}
      whileTap={{ scale: 0.985, y: -2 }}
      transition={{ type: "spring", stiffness: 320, damping: 22 }}
    />
  );
}
```

Guidance:

- hover should feel like lift, not a random jump
- tap should feel compressed and quick
- do not use giant hover movement on dense UI

## Accessibility and Performance

Use `useReducedMotion` for alternative behavior:

```tsx
import { motion, useReducedMotion } from "motion/react";

export function SafeReveal() {
  const reduceMotion = useReducedMotion();

  return (
    <motion.div
      initial={reduceMotion ? { opacity: 0 } : { opacity: 0, y: 20 }}
      animate={reduceMotion ? { opacity: 1 } : { opacity: 1, y: 0 }}
    />
  );
}
```

Practical rules:

- animate transform and opacity first
- keep blur and filter animation small and rare
- avoid continuous animation on huge surfaces
- pause ambient motion when it adds no information

## Default Expensive-Feeling Motion

Good starting points:

- reveal tween: `duration: 0.45–0.65`, `ease: [0.22, 1, 0.36, 1]`
- surface spring: `stiffness: 220–360`, `damping: 22–32`
- stagger gap: `0.05–0.1`

The goal is weighted, confident motion.
Not bouncy toy motion.
