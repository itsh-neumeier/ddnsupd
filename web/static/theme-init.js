try {
  const theme = localStorage.getItem('ddnsupd-theme');
  document.documentElement.classList.toggle(
    'dark',
    theme
      ? theme === 'dark'
      : matchMedia('(prefers-color-scheme: dark)').matches
  );
} catch {
  /* Storage can be unavailable in private contexts. */
}
