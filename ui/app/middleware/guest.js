export default defineNuxtRouteMiddleware(() => {
    const { getToken } = useAuthToken();
    const token = getToken();
    if (token) {
        return navigateTo('/');
    }
});