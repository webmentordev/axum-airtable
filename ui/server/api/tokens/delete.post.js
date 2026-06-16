export default defineEventHandler(async (event) => {
  const apiUrl = useRuntimeConfig(event).apiUrl;
  const body = await readBody(event);
  try {
    const data = await $fetch(`${apiUrl}/system/tokens/token/${body.app}`, {
        method: "DELETE",
        headers: {
            "Authorization": `Bearer ${body.token}`,
            "Content-Type": "application/json",
            "Accept": "application/json"
        },
        body: {
          unique_id: body.token_id
        }
    });
    return data;
  } catch (e) {
    console.log(e);
    throw createError({
      statusCode: e.response?.status || 500,
      statusMessage: e.data.message || 'Token delete failed'
    });
  }
});