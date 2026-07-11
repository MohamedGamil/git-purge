import { createRouter, createWebHistory } from 'vue-router';
import DashboardView from '../views/DashboardView.vue';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'dashboard',
      component: DashboardView,
    },
    {
      path: '/branches',
      name: 'branches',
      component: () => import('../views/DashboardView.vue'), // stubbed to DashboardView for Phase 4 foundation
    },
    {
      path: '/backups',
      name: 'backups',
      component: () => import('../views/DashboardView.vue'),
    },
    {
      path: '/diff',
      name: 'diff',
      component: () => import('../views/DashboardView.vue'),
    },
    {
      path: '/history',
      name: 'history',
      component: () => import('../views/DashboardView.vue'),
    },
    {
      path: '/auth',
      name: 'auth',
      component: () => import('../views/DashboardView.vue'),
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('../views/DashboardView.vue'),
    },
  ],
});

export default router;
