/*
 * C Writer for Rust Compatibility Testing
 *
 * This program generates Exodus II files using the C libexodus library.
 * The generated files are then read and validated by Rust programs.
 *
 * Usage: ./writer <test_case_name>
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include "exodusII.h"

#define OUTPUT_DIR "output"

/* Generate basic 2D mesh */
void generate_basic_2d(const char *filename) {
    int exoid, error;
    int cpu_word_size = 0;
    int io_word_size = 8;  /* Use double precision */

    /* Create file */
    exoid = ex_create(filename, EX_CLOBBER, &cpu_word_size, &io_word_size);
    if (exoid < 0) {
        fprintf(stderr, "Error: Could not create file %s\n", filename);
        exit(1);
    }

    /* Initialize */
    int num_dim = 2;
    int num_nodes = 4;
    int num_elem = 1;
    int num_elem_blk = 1;
    int num_node_sets = 0;
    int num_side_sets = 0;

    error = ex_put_init(exoid, "C-generated 2D mesh for Rust compatibility test",
                        num_dim, num_nodes, num_elem, num_elem_blk,
                        num_node_sets, num_side_sets);

    /* Coordinates */
    double x[4] = {0.0, 1.0, 1.0, 0.0};
    double y[4] = {0.0, 0.0, 1.0, 1.0};
    ex_put_coord(exoid, x, y, NULL);

    /* Coordinate names */
    char *coord_names[2] = {"x", "y"};
    ex_put_coord_names(exoid, coord_names);

    /* Element block */
    error = ex_put_block(exoid, EX_ELEM_BLOCK, 1, "QUAD4", 1, 4, 0, 0, 0);

    /* Connectivity */
    int connect[4] = {1, 2, 3, 4};
    ex_put_conn(exoid, EX_ELEM_BLOCK, 1, connect, NULL, NULL);

    /* QA record */
    char *qa_record[1][4];
    qa_record[0][0] = "exodus-c-writer";
    qa_record[0][1] = "1.0";

    time_t now = time(NULL);
    struct tm *t = localtime(&now);
    static char date_str[32];
    static char time_str[32];
    strftime(date_str, sizeof(date_str), "%Y-%m-%d", t);
    strftime(time_str, sizeof(time_str), "%H:%M:%S", t);

    qa_record[0][2] = date_str;
    qa_record[0][3] = time_str;
    ex_put_qa(exoid, 1, qa_record);

    ex_close(exoid);
    printf("Generated: %s\n", filename);
}

/* Generate basic 3D mesh */
void generate_basic_3d(const char *filename) {
    int exoid, error;
    int cpu_word_size = 0;
    int io_word_size = 8;

    exoid = ex_create(filename, EX_CLOBBER, &cpu_word_size, &io_word_size);
    if (exoid < 0) {
        fprintf(stderr, "Error: Could not create file %s\n", filename);
        exit(1);
    }

    int num_dim = 3;
    int num_nodes = 8;
    int num_elem = 1;
    int num_elem_blk = 1;
    int num_node_sets = 0;
    int num_side_sets = 0;

    error = ex_put_init(exoid, "C-generated 3D mesh for Rust compatibility test",
                        num_dim, num_nodes, num_elem, num_elem_blk,
                        num_node_sets, num_side_sets);

    /* Unit cube */
    double x[8] = {0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0};
    double y[8] = {0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0};
    double z[8] = {0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0};
    ex_put_coord(exoid, x, y, z);

    char *coord_names[3] = {"x", "y", "z"};
    ex_put_coord_names(exoid, coord_names);

    error = ex_put_block(exoid, EX_ELEM_BLOCK, 1, "HEX8", 1, 8, 0, 0, 0);

    int connect[8] = {1, 2, 3, 4, 5, 6, 7, 8};
    ex_put_conn(exoid, EX_ELEM_BLOCK, 1, connect, NULL, NULL);

    /* QA record */
    char *qa_record[1][4];
    qa_record[0][0] = "exodus-c-writer";
    qa_record[0][1] = "1.0";

    time_t now = time(NULL);
    struct tm *t = localtime(&now);
    static char date_str[32];
    static char time_str[32];
    strftime(date_str, sizeof(date_str), "%Y-%m-%d", t);
    strftime(time_str, sizeof(time_str), "%H:%M:%S", t);

    qa_record[0][2] = date_str;
    qa_record[0][3] = time_str;
    ex_put_qa(exoid, 1, qa_record);

    ex_close(exoid);
    printf("Generated: %s\n", filename);
}

/* Generate mesh with variables */
void generate_with_variables(const char *filename) {
    int exoid, error;
    int cpu_word_size = 0;
    int io_word_size = 8;

    exoid = ex_create(filename, EX_CLOBBER, &cpu_word_size, &io_word_size);
    if (exoid < 0) {
        fprintf(stderr, "Error: Could not create file %s\n", filename);
        exit(1);
    }

    int num_dim = 2;
    int num_nodes = 4;
    int num_elem = 1;
    int num_elem_blk = 1;
    int num_node_sets = 0;
    int num_side_sets = 0;

    error = ex_put_init(exoid, "C-generated mesh with variables",
                        num_dim, num_nodes, num_elem, num_elem_blk,
                        num_node_sets, num_side_sets);

    double x[4] = {0.0, 1.0, 1.0, 0.0};
    double y[4] = {0.0, 0.0, 1.0, 1.0};
    ex_put_coord(exoid, x, y, NULL);

    char *coord_names[2] = {"x", "y"};
    ex_put_coord_names(exoid, coord_names);

    error = ex_put_block(exoid, EX_ELEM_BLOCK, 1, "QUAD4", 1, 4, 0, 0, 0);

    int connect[4] = {1, 2, 3, 4};
    ex_put_conn(exoid, EX_ELEM_BLOCK, 1, connect, NULL, NULL);

    /* Add variables */
    int num_glo_vars = 1;
    ex_put_variable_param(exoid, EX_GLOBAL, num_glo_vars);
    char *glo_var_names[1] = {"time_value"};
    ex_put_variable_names(exoid, EX_GLOBAL, num_glo_vars, glo_var_names);

    int num_nod_vars = 1;
    ex_put_variable_param(exoid, EX_NODAL, num_nod_vars);
    char *nod_var_names[1] = {"temperature"};
    ex_put_variable_names(exoid, EX_NODAL, num_nod_vars, nod_var_names);

    /* Write 2 time steps */
    for (int step = 0; step < 2; step++) {
        double time_value = step * 0.1;
        ex_put_time(exoid, step + 1, &time_value);

        /* Global variable */
        double glo_vals[1] = {time_value};
        ex_put_var(exoid, step + 1, EX_GLOBAL, 1, 1, 1, glo_vals);

        /* Nodal variable */
        double nod_vals[4] = {
            100.0 + step * 10.0,
            110.0 + step * 10.0,
            120.0 + step * 10.0,
            130.0 + step * 10.0
        };
        ex_put_var(exoid, step + 1, EX_NODAL, 1, 1, num_nodes, nod_vals);
    }

    /* QA record */
    char *qa_record[1][4];
    qa_record[0][0] = "exodus-c-writer";
    qa_record[0][1] = "1.0";

    time_t now = time(NULL);
    struct tm *t = localtime(&now);
    static char date_str[32];
    static char time_str[32];
    strftime(date_str, sizeof(date_str), "%Y-%m-%d", t);
    strftime(time_str, sizeof(time_str), "%H:%M:%S", t);

    qa_record[0][2] = date_str;
    qa_record[0][3] = time_str;
    ex_put_qa(exoid, 1, qa_record);

    ex_close(exoid);
    printf("Generated: %s\n", filename);
}

/* Generate mesh with multiple element blocks */
void generate_multiple_blocks(const char *filename) {
    int exoid, error;
    int cpu_word_size = 0;
    int io_word_size = 8;

    exoid = ex_create(filename, EX_CLOBBER, &cpu_word_size, &io_word_size);
    if (exoid < 0) {
        fprintf(stderr, "Error: Could not create file %s\n", filename);
        exit(1);
    }

    int num_dim = 2;
    int num_nodes = 8;
    int num_elem = 3;
    int num_elem_blk = 2;  /* Two blocks: quads and tris */
    int num_node_sets = 0;
    int num_side_sets = 0;

    error = ex_put_init(exoid, "C-generated multi-block mesh",
                        num_dim, num_nodes, num_elem, num_elem_blk,
                        num_node_sets, num_side_sets);

    /* Coordinates for a small mesh */
    double x[8] = {0.0, 1.0, 2.0, 0.0, 1.0, 2.0, 0.5, 1.5};
    double y[8] = {0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.5, 1.5};
    ex_put_coord(exoid, x, y, NULL);

    char *coord_names[2] = {"x", "y"};
    ex_put_coord_names(exoid, coord_names);

    /* Block 1: Two quads */
    error = ex_put_block(exoid, EX_ELEM_BLOCK, 10, "QUAD4", 2, 4, 0, 0, 0);
    int connect_quad[8] = {1, 2, 5, 4,  /* element 1 */
                           2, 3, 6, 5}; /* element 2 */
    ex_put_conn(exoid, EX_ELEM_BLOCK, 10, connect_quad, NULL, NULL);

    /* Block 2: One triangle */
    error = ex_put_block(exoid, EX_ELEM_BLOCK, 20, "TRI3", 1, 3, 0, 0, 0);
    int connect_tri[3] = {4, 7, 8};
    ex_put_conn(exoid, EX_ELEM_BLOCK, 20, connect_tri, NULL, NULL);

    /* QA record */
    char *qa_record[1][4];
    qa_record[0][0] = "exodus-c-writer";
    qa_record[0][1] = "1.0";

    time_t now = time(NULL);
    struct tm *t = localtime(&now);
    static char date_str[32];
    static char time_str[32];
    strftime(date_str, sizeof(date_str), "%Y-%m-%d", t);
    strftime(time_str, sizeof(time_str), "%H:%M:%S", t);

    qa_record[0][2] = date_str;
    qa_record[0][3] = time_str;
    ex_put_qa(exoid, 1, qa_record);

    ex_close(exoid);
    printf("Generated: %s\n", filename);
}

/* Generate mesh with node sets */
void generate_with_node_sets(const char *filename) {
    int exoid, error;
    int cpu_word_size = 0;
    int io_word_size = 8;

    exoid = ex_create(filename, EX_CLOBBER, &cpu_word_size, &io_word_size);
    if (exoid < 0) {
        fprintf(stderr, "Error: Could not create file %s\n", filename);
        exit(1);
    }

    int num_dim = 2;
    int num_nodes = 4;
    int num_elem = 1;
    int num_elem_blk = 1;
    int num_node_sets = 2;  /* Two node sets */
    int num_side_sets = 0;

    error = ex_put_init(exoid, "C-generated mesh with node sets",
                        num_dim, num_nodes, num_elem, num_elem_blk,
                        num_node_sets, num_side_sets);

    /* Basic quad mesh */
    double x[4] = {0.0, 1.0, 1.0, 0.0};
    double y[4] = {0.0, 0.0, 1.0, 1.0};
    ex_put_coord(exoid, x, y, NULL);

    char *coord_names[2] = {"x", "y"};
    ex_put_coord_names(exoid, coord_names);

    error = ex_put_block(exoid, EX_ELEM_BLOCK, 1, "QUAD4", 1, 4, 0, 0, 0);
    int connect[4] = {1, 2, 3, 4};
    ex_put_conn(exoid, EX_ELEM_BLOCK, 1, connect, NULL, NULL);

    /* Node set 1: bottom edge */
    ex_put_set_param(exoid, EX_NODE_SET, 100, 2, 0);
    int ns1_nodes[2] = {1, 2};
    ex_put_set(exoid, EX_NODE_SET, 100, ns1_nodes, NULL);

    /* Node set 2: right edge */
    ex_put_set_param(exoid, EX_NODE_SET, 200, 2, 0);
    int ns2_nodes[2] = {2, 3};
    ex_put_set(exoid, EX_NODE_SET, 200, ns2_nodes, NULL);

    /* QA record */
    char *qa_record[1][4];
    qa_record[0][0] = "exodus-c-writer";
    qa_record[0][1] = "1.0";

    time_t now = time(NULL);
    struct tm *t = localtime(&now);
    static char date_str[32];
    static char time_str[32];
    strftime(date_str, sizeof(date_str), "%Y-%m-%d", t);
    strftime(time_str, sizeof(time_str), "%H:%M:%S", t);

    qa_record[0][2] = date_str;
    qa_record[0][3] = time_str;
    ex_put_qa(exoid, 1, qa_record);

    ex_close(exoid);
    printf("Generated: %s\n", filename);
}

/* Generate mesh with side sets */
void generate_with_side_sets(const char *filename) {
    int exoid, error;
    int cpu_word_size = 0;
    int io_word_size = 8;

    exoid = ex_create(filename, EX_CLOBBER, &cpu_word_size, &io_word_size);
    if (exoid < 0) {
        fprintf(stderr, "Error: Could not create file %s\n", filename);
        exit(1);
    }

    int num_dim = 2;
    int num_nodes = 4;
    int num_elem = 1;
    int num_elem_blk = 1;
    int num_node_sets = 0;
    int num_side_sets = 2;  /* Two side sets */

    error = ex_put_init(exoid, "C-generated mesh with side sets",
                        num_dim, num_nodes, num_elem, num_elem_blk,
                        num_node_sets, num_side_sets);

    /* Basic quad mesh */
    double x[4] = {0.0, 1.0, 1.0, 0.0};
    double y[4] = {0.0, 0.0, 1.0, 1.0};
    ex_put_coord(exoid, x, y, NULL);

    char *coord_names[2] = {"x", "y"};
    ex_put_coord_names(exoid, coord_names);

    error = ex_put_block(exoid, EX_ELEM_BLOCK, 1, "QUAD4", 1, 4, 0, 0, 0);
    int connect[4] = {1, 2, 3, 4};
    ex_put_conn(exoid, EX_ELEM_BLOCK, 1, connect, NULL, NULL);

    /* Side set 1: bottom edge (side 1 of element 1) */
    ex_put_set_param(exoid, EX_SIDE_SET, 100, 1, 0);
    int ss1_elem[1] = {1};
    int ss1_side[1] = {1};
    ex_put_set(exoid, EX_SIDE_SET, 100, ss1_elem, ss1_side);

    /* Side set 2: right edge (side 2 of element 1) */
    ex_put_set_param(exoid, EX_SIDE_SET, 200, 1, 0);
    int ss2_elem[1] = {1};
    int ss2_side[1] = {2};
    ex_put_set(exoid, EX_SIDE_SET, 200, ss2_elem, ss2_side);

    /* QA record */
    char *qa_record[1][4];
    qa_record[0][0] = "exodus-c-writer";
    qa_record[0][1] = "1.0";

    time_t now = time(NULL);
    struct tm *t = localtime(&now);
    static char date_str[32];
    static char time_str[32];
    strftime(date_str, sizeof(date_str), "%Y-%m-%d", t);
    strftime(time_str, sizeof(time_str), "%H:%M:%S", t);

    qa_record[0][2] = date_str;
    qa_record[0][3] = time_str;
    ex_put_qa(exoid, 1, qa_record);

    ex_close(exoid);
    printf("Generated: %s\n", filename);
}

/* Generate comprehensive test with multiple features */
void generate_comprehensive(const char *filename) {
    int exoid, error;
    int cpu_word_size = 0;
    int io_word_size = 8;

    exoid = ex_create(filename, EX_CLOBBER, &cpu_word_size, &io_word_size);
    if (exoid < 0) {
        fprintf(stderr, "Error: Could not create file %s\n", filename);
        exit(1);
    }

    int num_dim = 2;
    int num_nodes = 6;
    int num_elem = 2;
    int num_elem_blk = 1;
    int num_node_sets = 1;
    int num_side_sets = 1;

    error = ex_put_init(exoid, "C-generated comprehensive test mesh",
                        num_dim, num_nodes, num_elem, num_elem_blk,
                        num_node_sets, num_side_sets);

    /* 2x1 quad mesh */
    double x[6] = {0.0, 1.0, 2.0, 0.0, 1.0, 2.0};
    double y[6] = {0.0, 0.0, 0.0, 1.0, 1.0, 1.0};
    ex_put_coord(exoid, x, y, NULL);

    char *coord_names[2] = {"x", "y"};
    ex_put_coord_names(exoid, coord_names);

    /* Element block */
    error = ex_put_block(exoid, EX_ELEM_BLOCK, 1, "QUAD4", 2, 4, 0, 0, 0);
    int connect[8] = {1, 2, 5, 4,   /* element 1 */
                      2, 3, 6, 5};  /* element 2 */
    ex_put_conn(exoid, EX_ELEM_BLOCK, 1, connect, NULL, NULL);

    /* Node set: left edge */
    ex_put_set_param(exoid, EX_NODE_SET, 100, 2, 0);
    int ns_nodes[2] = {1, 4};
    ex_put_set(exoid, EX_NODE_SET, 100, ns_nodes, NULL);

    /* Side set: bottom edge */
    ex_put_set_param(exoid, EX_SIDE_SET, 200, 2, 0);
    int ss_elem[2] = {1, 2};
    int ss_side[2] = {1, 1};
    ex_put_set(exoid, EX_SIDE_SET, 200, ss_elem, ss_side);

    /* Add variables */
    int num_nod_vars = 1;
    ex_put_variable_param(exoid, EX_NODAL, num_nod_vars);
    char *nod_var_names[1] = {"temperature"};
    ex_put_variable_names(exoid, EX_NODAL, num_nod_vars, nod_var_names);

    /* Write 2 time steps */
    for (int step = 0; step < 2; step++) {
        double time_value = step * 0.5;
        ex_put_time(exoid, step + 1, &time_value);

        double nod_vals[6] = {
            100.0 + step * 10.0,
            110.0 + step * 10.0,
            120.0 + step * 10.0,
            130.0 + step * 10.0,
            140.0 + step * 10.0,
            150.0 + step * 10.0
        };
        ex_put_var(exoid, step + 1, EX_NODAL, 1, 1, num_nodes, nod_vals);
    }

    /* QA record */
    char *qa_record[1][4];
    qa_record[0][0] = "exodus-c-writer";
    qa_record[0][1] = "1.0";

    time_t now = time(NULL);
    struct tm *t = localtime(&now);
    static char date_str[32];
    static char time_str[32];
    strftime(date_str, sizeof(date_str), "%Y-%m-%d", t);
    strftime(time_str, sizeof(time_str), "%H:%M:%S", t);

    qa_record[0][2] = date_str;
    qa_record[0][3] = time_str;
    ex_put_qa(exoid, 1, qa_record);

    ex_close(exoid);
    printf("Generated: %s\n", filename);
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        fprintf(stderr, "Usage: %s <test_case>\n", argv[0]);
        fprintf(stderr, "Test cases:\n");
        fprintf(stderr, "  basic_2d          - Simple 2D quad mesh\n");
        fprintf(stderr, "  basic_3d          - Simple 3D hex mesh\n");
        fprintf(stderr, "  with_variables    - Mesh with time-dependent variables\n");
        fprintf(stderr, "  multiple_blocks   - Mesh with multiple element blocks\n");
        fprintf(stderr, "  with_node_sets    - Mesh with node sets\n");
        fprintf(stderr, "  with_side_sets    - Mesh with side sets\n");
        fprintf(stderr, "  comprehensive     - Comprehensive test with all features\n");
        fprintf(stderr, "  all               - Generate all test cases\n");
        return 1;
    }

    /* Create output directory */
    system("mkdir -p " OUTPUT_DIR);

    const char *test_case = argv[1];
    char filename[256];
    int count = 0;

    if (strcmp(test_case, "basic_2d") == 0) {
        snprintf(filename, sizeof(filename), "%s/c_basic_2d.exo", OUTPUT_DIR);
        generate_basic_2d(filename);
    }
    else if (strcmp(test_case, "basic_3d") == 0) {
        snprintf(filename, sizeof(filename), "%s/c_basic_3d.exo", OUTPUT_DIR);
        generate_basic_3d(filename);
    }
    else if (strcmp(test_case, "with_variables") == 0) {
        snprintf(filename, sizeof(filename), "%s/c_with_variables.exo", OUTPUT_DIR);
        generate_with_variables(filename);
    }
    else if (strcmp(test_case, "multiple_blocks") == 0) {
        snprintf(filename, sizeof(filename), "%s/c_multiple_blocks.exo", OUTPUT_DIR);
        generate_multiple_blocks(filename);
    }
    else if (strcmp(test_case, "with_node_sets") == 0) {
        snprintf(filename, sizeof(filename), "%s/c_with_node_sets.exo", OUTPUT_DIR);
        generate_with_node_sets(filename);
    }
    else if (strcmp(test_case, "with_side_sets") == 0) {
        snprintf(filename, sizeof(filename), "%s/c_with_side_sets.exo", OUTPUT_DIR);
        generate_with_side_sets(filename);
    }
    else if (strcmp(test_case, "comprehensive") == 0) {
        snprintf(filename, sizeof(filename), "%s/c_comprehensive.exo", OUTPUT_DIR);
        generate_comprehensive(filename);
    }
    else if (strcmp(test_case, "all") == 0) {
        printf("Generating all C test files...\n\n");

        snprintf(filename, sizeof(filename), "%s/c_basic_2d.exo", OUTPUT_DIR);
        generate_basic_2d(filename);
        count++;

        snprintf(filename, sizeof(filename), "%s/c_basic_3d.exo", OUTPUT_DIR);
        generate_basic_3d(filename);
        count++;

        snprintf(filename, sizeof(filename), "%s/c_with_variables.exo", OUTPUT_DIR);
        generate_with_variables(filename);
        count++;

        snprintf(filename, sizeof(filename), "%s/c_multiple_blocks.exo", OUTPUT_DIR);
        generate_multiple_blocks(filename);
        count++;

        snprintf(filename, sizeof(filename), "%s/c_with_node_sets.exo", OUTPUT_DIR);
        generate_with_node_sets(filename);
        count++;

        snprintf(filename, sizeof(filename), "%s/c_with_side_sets.exo", OUTPUT_DIR);
        generate_with_side_sets(filename);
        count++;

        snprintf(filename, sizeof(filename), "%s/c_comprehensive.exo", OUTPUT_DIR);
        generate_comprehensive(filename);
        count++;

        printf("\n✓ All %d test files generated successfully!\n", count);
    }
    else {
        fprintf(stderr, "Unknown test case: %s\n", test_case);
        return 1;
    }

    return 0;
}
