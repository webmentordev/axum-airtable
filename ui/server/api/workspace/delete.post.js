export default defineEventHandler(async (event) => {
  const apiUrl = useRuntimeConfig(event).apiUrl;
  const body = await readBody(event);
  try {
    const data = await $fetch(`${apiUrl}/system/workspaces/${body.app_id}/${body.workspace_id}`, {
      method: "DELETE",
      headers: {
          "Authorization": `Bearer ${body.token}`,
          "Content-Type": "application/json",
          "Accept": "application/json"
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