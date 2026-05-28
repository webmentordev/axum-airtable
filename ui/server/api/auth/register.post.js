export default defineEventHandler(async (event) => {
  const apiUrl = useRuntimeConfig(event).apiUrl;
  const body = await readBody(event);
  try {
    const data = await $fetch(`${apiUrl}/auth/signup`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Accept": "application/json"
      },
      body: {
        name: body.name,
        username: body.username,
        email: body.email,
        password: body.password,
        confirm_password: body.confirm_password,
      }
    });
    return { message: data.message };
  } catch (e) {
    throw createError({
      statusCode: e.response?.status || 500,
      statusMessage: e.data.message || 'Signup failed'
    });
  }
});