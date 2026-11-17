<script lang="ts">
  let { name, size = 20 }: { name: string; size?: number } = $props();
  const paths: Record<string, string> = {
    dashboard: 'M3 3h7v7H3z M14 3h7v7h-7z M3 14h7v7H3z M14 14h7v7h-7z',
    records: 'M4 4h16v6H4z M4 14h16v6H4z M7 7h.01 M7 17h.01 M11 7h6 M11 17h6',
    providers: 'M8 3v5 M16 3v5 M6 8h12v4a6 6 0 0 1-12 0z M12 18v4',
    history: 'M3 12a9 9 0 1 0 3-6 M3 3v6h6 M12 7v5l3 2',
    administration:
      'M12 3a4 4 0 1 1 0 8 4 4 0 0 1 0-8 M4 22v-3a8 8 0 0 1 16 0v3',
    plus: 'M12 5v14 M5 12h14',
    check: 'M5 12l4 4L19 6',
    close: 'M6 6l12 12 M18 6L6 18',
    refresh: 'M20 7v5h-5 M4 17v-5h5 M6 6a8 8 0 0 1 14 6 M18 18A8 8 0 0 1 4 12',
    globe:
      'M3 12h18 M12 3c-5 5-5 13 0 18 M12 3c5 5 5 13 0 18 M3 12a9 9 0 1 0 18 0 9 9 0 0 0-18 0',
    sun: 'M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8 M12 1v3 M12 20v3 M1 12h3 M20 12h3 M4 4l2 2 M18 18l2 2 M4 20l2-2 M18 6l2-2',
    logout: 'M9 4H4v16h5 M10 12h11 M17 8l4 4-4 4',
    arrow: 'M5 12h14 M13 6l6 6-6 6',
    warning: 'M12 3L2 21h20z M12 9v5 M12 17h.01',
    lock: 'M6 10h12v11H6z M8 10V6a4 4 0 0 1 8 0v4'
  };
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.65"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"><path d={paths[name] ?? paths.globe} /></svg
>
