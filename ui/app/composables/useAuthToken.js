export const useAuthToken = () => {
    const token = useCookie('at_auth_token', {
        maxAge: 3600 * 60,
        sameSite: 'strict',
        secure: import.meta.client ? window.location.protocol === 'https:' : true,
        path: '/'
    });
    
    const setToken = (newToken) => {
        token.value = newToken;
    };
    const removeToken = () => {
        token.value = null;
    };
    const getToken = () => {
        return token.value;
    };
    return {
        token,
        setToken,
        removeToken,
        getToken
    };
};