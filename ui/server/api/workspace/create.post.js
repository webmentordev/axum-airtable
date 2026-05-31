export default defineEventHandler(async (event) => {
  const apiUrl = useRuntimeConfig(event).apiUrl;
  const body = await readBody(event);
  try {
    const data = await $fetch(`${apiUrl}/system/workspaces/${body.app_id}`, {
      method: "POST",
      headers: {
          "Authorization": `Bearer ${body.token}`,
          "Content-Type": "application/json",
          "Accept": "application/json"
      },
      body: {
        title: body.title,
        position: 0
      }
    });
    return data;
  } catch (e) {
    console.log(e);
    throw createError({
      statusCode: e.response?.status || 500,
      statusMessage: e.data.message || 'Workspace create failed'
    });
  }
});