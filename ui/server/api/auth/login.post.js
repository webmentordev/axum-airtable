export default defineEventHandler(async (event) => {
  const apiUrl = useRuntimeConfig(event).apiUrl;
  const body = await readBody(event);
  try {
    const data = await $fetch(`${apiUrl}/auth/login`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Accept": "application/json"
      },
      body: {
        email: body.email,
        password: body.password
      }
    });
    return { token: data.token };
  } catch (e) {
    throw createError({
      statusCode: e.response?.status || 500,
      statusMessage: e.data.message || 'Login failed'
    });
  }
});