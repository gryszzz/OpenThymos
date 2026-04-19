/* THYMOS — site interactions (light, no deps) */

(() => {
  // Mobile nav
  const nav = document.querySelector('header.nav');
  const toggle = document.getElementById('nav-toggle');
  if (toggle && nav) {
    toggle.addEventListener('click', () => {
      const open = nav.classList.toggle('is-open');
      toggle.setAttribute('aria-expanded', open ? 'true' : 'false');
    });
  }

  // Reveal-on-scroll
  const revealEls = document.querySelectorAll('.reveal');
  if (revealEls.length && 'IntersectionObserver' in window) {
    const io = new IntersectionObserver(
      (entries) => {
        entries.forEach((e) => {
          if (e.isIntersecting) {
            e.target.classList.add('is-visible');
            io.unobserve(e.target);
          }
        });
      },
      { threshold: 0.12 }
    );
    revealEls.forEach((el) => io.observe(el));
  } else {
    revealEls.forEach((el) => el.classList.add('is-visible'));
  }

  // Typewriter for terminal demo
  const term = document.querySelector('[data-type]');
  if (term) {
    const lines = JSON.parse(term.dataset.type);
    const speed = Number(term.dataset.speed || 22);
    term.textContent = '';
    let i = 0, j = 0;

    const tick = () => {
      if (i >= lines.length) {
        const c = document.createElement('span');
        c.className = 'terminal-cursor';
        term.appendChild(c);
        return;
      }
      const line = lines[i];
      if (j === 0) {
        const row = document.createElement('div');
        row.className = line.cls || '';
        row.dataset.idx = String(i);
        term.appendChild(row);
      }
      const row = term.querySelector(`[data-idx="${i}"]`);
      row.textContent += line.text[j];
      j++;
      if (j >= line.text.length) {
        i++;
        j = 0;
        setTimeout(tick, line.pause || 140);
      } else {
        setTimeout(tick, speed);
      }
    };
    // Start when visible
    if ('IntersectionObserver' in window) {
      const obs = new IntersectionObserver((es) => {
        if (es.some((e) => e.isIntersecting)) {
          tick();
          obs.disconnect();
        }
      }, { threshold: 0.2 });
      obs.observe(term);
    } else {
      tick();
    }
  }

  // Copy buttons on code blocks
  document.querySelectorAll('.prose pre').forEach((pre) => {
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.className = 'copy-btn';
    btn.textContent = 'Copy';
    btn.style.cssText = `
      position: absolute; top: 10px; right: 10px;
      height: 26px; padding: 0 10px;
      font-size: 11px; letter-spacing: .08em; text-transform: uppercase;
      font-family: var(--font-mono);
      color: var(--text-dim);
      background: rgba(255,255,255,0.04);
      border: 1px solid rgba(255,255,255,0.08);
      border-radius: 6px;
      cursor: pointer;
      opacity: 0; transition: opacity .18s ease, color .18s ease;
    `;
    pre.style.position = 'relative';
    pre.appendChild(btn);
    pre.addEventListener('mouseenter', () => (btn.style.opacity = '1'));
    pre.addEventListener('mouseleave', () => (btn.style.opacity = '0'));
    btn.addEventListener('click', async () => {
      const text = pre.querySelector('code')?.innerText || pre.innerText;
      try {
        await navigator.clipboard.writeText(text);
        btn.textContent = 'Copied';
        btn.style.color = 'var(--accent)';
        setTimeout(() => {
          btn.textContent = 'Copy';
          btn.style.color = 'var(--text-dim)';
        }, 1400);
      } catch {
        btn.textContent = 'Err';
      }
    });
  });
})();
