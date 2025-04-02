import { createRouter, createWebHistory } from 'vue-router';

const routes = [
  {
    path: '/',
    name: 'Home',
    component: () => import('../views/HomePage.vue')
  },
  {
    path: '/preview/:id',
    name: 'Preview',
    component: () => import('../views/Preview.vue'),
    props: true
  }
];

const router = createRouter({
  history: createWebHistory(),
  routes
});

export default router;