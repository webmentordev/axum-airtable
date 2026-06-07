export default defineEventHandler(async (event) => {
  const apiUrl = useRuntimeConfig(event).apiUrl;
  const body = await readBody(event);
  try {
    const data = await $fetch(`${apiUrl}/system/fields/${body.workspace}`, {
        method: "POST",
        headers: {
            "Authorization": `Bearer ${body.token}`,
            "Content-Type": "application/json",
            "Accept": "application/json"
        },
        body: {
            title: body.title,
            field_type: body.field_type,
            position: 0
        }
    });
    return data;
  } catch (e) {
    throw createError({
      statusCode: e.response?.status || 500,
      statusMessage: e.data.message || 'Field create failed'
    });
  }
});