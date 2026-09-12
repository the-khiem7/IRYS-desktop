import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import mermaid from 'astro-mermaid';

export default defineConfig({
  output: 'static',
  site: 'https://the-khiem7.github.io',
  base: '/IRYS-desktop',
  publicDir: '../docs/img',
  vite: {
    cacheDir: '../node_modules/.vite-docs',
  },
  integrations: [
    mermaid({ autoTheme: true, enableLog: false }),
    starlight({
      title: 'IRYS Documentation',
      description: 'User and contributor documentation for the IRYS eye-break desktop app.',
      social: [
        {
          icon: 'github',
          label: 'GitHub',
          href: 'https://github.com/the-khiem7/IRYS-desktop',
        },
      ],
      markdown: {
        processedDirs: ['../docs'],
      },
      sidebar: [
        {
          label: 'User Guide',
          items: [
            { label: 'Overview', slug: 'user' },
            { label: 'Install and first run', slug: 'user/install-first-run' },
            { label: 'Everyday use', slug: 'user/everyday-use' },
            { label: 'Settings', slug: 'user/settings' },
            { label: 'Privacy and safety', slug: 'user/privacy' },
            { label: 'Troubleshooting', slug: 'user/troubleshooting' },
          ],
        },
        {
          label: 'Contributor Guide',
          items: [
            { label: 'Project scope and current truth', slug: 'contributors/overview' },
            { label: 'Architecture and execution flow', slug: 'contributors/architecture' },
            { label: 'Contracts and procedures', slug: 'contributors/contracts' },
            { label: 'Roadmap', slug: 'contributors/roadmap' },
            { label: 'Decisions and open questions', slug: 'contributors/decisions' },
            { label: 'Docker build environment', slug: 'runbooks/docker-build-environment' },
          ],
        },
      ],
    }),
  ],
});
