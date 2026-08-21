import { Routes } from '@angular/router';

export default [
  {
    path: 'stylist',
    loadComponent: () =>
      import('./stylist-chat.component').then(m => m.StylistChatComponent),
  },
] satisfies Routes;
