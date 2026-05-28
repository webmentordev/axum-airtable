export default defineEventHandler(async (event) => {
  const apiUrl = useRuntimeConfig(event).apiUrl;
  try {
    const data = await $fetch(`${apiUrl}/health`);
    return {
      status: 200,
      message: data.message
    };
  } catch (e) {
    throw createError({
      statusCode: e.response?.status || 500,
      statusMessage: e.data.message || 'Health failed'
    });
  }
});